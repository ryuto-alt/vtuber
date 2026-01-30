/// Native RTMP server — FFmpegを使わずRustで直接RTMP→FLV変換
///
/// OBSからのRTMP接続を受け付け、FLVフォーマットでbroadcastチャンネルに流す。
/// RTMPとFLVはほぼ同じデータ形式のため、変換は軽量。
use bytes::{Bytes, BytesMut, BufMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tracing::{info, debug};

use std::collections::HashMap;

use crate::streaming::StreamManager;

// ───────── FLV helpers ─────────

fn flv_file_header() -> Bytes {
    let mut b = BytesMut::with_capacity(13);
    b.put_slice(b"FLV");  // signature
    b.put_u8(1);           // version
    b.put_u8(0x05);        // flags: audio + video
    b.put_u32(9);           // header size
    b.put_u32(0);           // previous tag size 0
    b.freeze()
}

fn flv_tag(tag_type: u8, timestamp: u32, data: &[u8]) -> Bytes {
    let data_len = data.len() as u32;
    let tag_total = 11 + data_len;
    let mut b = BytesMut::with_capacity((tag_total + 4) as usize);
    b.put_u8(tag_type);
    // data size (3 bytes BE)
    b.put_u8((data_len >> 16) as u8);
    b.put_u8((data_len >> 8) as u8);
    b.put_u8(data_len as u8);
    // timestamp (3 bytes BE) + extension (1 byte)
    b.put_u8((timestamp >> 16) as u8);
    b.put_u8((timestamp >> 8) as u8);
    b.put_u8(timestamp as u8);
    b.put_u8((timestamp >> 24) as u8);
    // stream id (3 bytes, always 0)
    b.put_u8(0); b.put_u8(0); b.put_u8(0);
    b.put_slice(data);
    b.put_u32(tag_total); // previous tag size
    b.freeze()
}

// ───────── RTMP chunk state ─────────

#[derive(Default)]
struct ChunkState {
    timestamp: u32,
    msg_length: u32,
    msg_type_id: u8,
    msg_stream_id: u32,
    buf: BytesMut,
    bytes_left: u32,
    // timestamp deltaを追跡（fmt=3で再利用）
    last_delta: u32,
    has_extended_ts: bool,
}

// ───────── Public entry point ─────────

pub async fn run_native_rtmp(
    port: u16,
    sender: broadcast::Sender<Bytes>,
    stream_manager: StreamManager,
    stream_id: String,
) -> Result<(), String> {
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await
        .map_err(|e| format!("Bind port {}: {}", port, e))?;
    info!("[NativeRTMP] Listening on {}", addr);

    // 1接続のみ受け付け
    let (tcp, peer) = listener.accept().await
        .map_err(|e| format!("Accept: {}", e))?;
    info!("[NativeRTMP] Connection from {}", peer);
    drop(listener);

    let (rd, wr) = tcp.into_split();
    let mut r = BufReader::with_capacity(65536, rd);
    let mut w = BufWriter::with_capacity(65536, wr);

    // ──── Handshake ────
    let c0 = r.read_u8().await.map_err(|e| format!("C0: {}", e))?;
    if c0 != 3 { return Err(format!("Unsupported RTMP version: {}", c0)); }

    let mut c1 = vec![0u8; 1536];
    r.read_exact(&mut c1).await.map_err(|e| format!("C1: {}", e))?;

    // S0 + S1 + S2
    let s1 = vec![0u8; 1536]; // simple zero S1
    let mut s2 = c1.clone();
    s2[4..8].copy_from_slice(&[0; 4]); // time2
    w.write_u8(3).await.map_err(|e| format!("S0: {}", e))?;
    w.write_all(&s1).await.map_err(|e| format!("S1: {}", e))?;
    w.write_all(&s2).await.map_err(|e| format!("S2: {}", e))?;
    w.flush().await.map_err(|e| format!("flush: {}", e))?;

    let mut c2 = vec![0u8; 1536];
    r.read_exact(&mut c2).await.map_err(|e| format!("C2: {}", e))?;
    info!("[NativeRTMP] Handshake complete");

    // ──── Chunk stream loop ────
    let mut chunk_size: u32 = 128;
    let mut chunks: HashMap<u32, ChunkState> = HashMap::new();
    let mut flv_started = false;
    let mut headers_done = false; // ヘッダー収集完了フラグ（ロック回避用）
    let mut total_recv: u64 = 0;
    let mut ack_window: u32 = 2500000;
    let mut last_ack: u64 = 0;
    let t0 = std::time::Instant::now();
    let mut gop_buf: Vec<Bytes> = Vec::with_capacity(300); // GOPバッファ（最新キーフレーム〜）
    let mut got_keyframe = false;

    loop {
        let first = match r.read_u8().await {
            Ok(b) => b,
            Err(_) => { info!("[NativeRTMP] Connection closed"); break; }
        };
        total_recv += 1;

        let fmt = (first >> 6) & 3;
        let cs_id = match first & 0x3F {
            0 => { let b = r.read_u8().await.map_err(|e| format!("cs: {}", e))?; total_recv += 1; b as u32 + 64 }
            1 => {
                let mut b = [0u8; 2];
                r.read_exact(&mut b).await.map_err(|e| format!("cs: {}", e))?;
                total_recv += 2;
                (b[1] as u32) * 256 + b[0] as u32 + 64
            }
            n => n as u32,
        };

        let st = chunks.entry(cs_id).or_default();

        // Message header
        match fmt {
            0 => {
                let mut h = [0u8; 11];
                r.read_exact(&mut h).await.map_err(|e| format!("h0: {}", e))?;
                total_recv += 11;
                let ts = read_u24_be(&h[0..3]);
                st.msg_length = read_u24_be(&h[3..6]);
                st.msg_type_id = h[6];
                st.msg_stream_id = u32::from_le_bytes([h[7], h[8], h[9], h[10]]);
                if ts == 0xFFFFFF {
                    st.timestamp = read_u32_be_async(&mut r).await?;
                    st.has_extended_ts = true;
                    total_recv += 4;
                } else {
                    st.timestamp = ts;
                    st.has_extended_ts = false;
                }
                st.last_delta = 0;
                st.buf.clear();
                st.bytes_left = st.msg_length;
            }
            1 => {
                let mut h = [0u8; 7];
                r.read_exact(&mut h).await.map_err(|e| format!("h1: {}", e))?;
                total_recv += 7;
                let delta = read_u24_be(&h[0..3]);
                st.msg_length = read_u24_be(&h[3..6]);
                st.msg_type_id = h[6];
                if delta == 0xFFFFFF {
                    let ext = read_u32_be_async(&mut r).await?;
                    total_recv += 4;
                    st.timestamp += ext;
                    st.last_delta = ext;
                    st.has_extended_ts = true;
                } else {
                    st.timestamp += delta;
                    st.last_delta = delta;
                    st.has_extended_ts = false;
                }
                st.buf.clear();
                st.bytes_left = st.msg_length;
            }
            2 => {
                let mut h = [0u8; 3];
                r.read_exact(&mut h).await.map_err(|e| format!("h2: {}", e))?;
                total_recv += 3;
                let delta = read_u24_be(&h[0..3]);
                if delta == 0xFFFFFF {
                    let ext = read_u32_be_async(&mut r).await?;
                    total_recv += 4;
                    st.timestamp += ext;
                    st.last_delta = ext;
                    st.has_extended_ts = true;
                } else {
                    st.timestamp += delta;
                    st.last_delta = delta;
                    st.has_extended_ts = false;
                }
                if st.bytes_left == 0 {
                    st.buf.clear();
                    st.bytes_left = st.msg_length;
                }
            }
            3 => {
                // 前のチャンクと同じ — extended timestamp の場合は4バイト読む
                if st.has_extended_ts {
                    let ext = read_u32_be_async(&mut r).await?;
                    total_recv += 4;
                    if st.bytes_left == 0 {
                        // 新しいメッセージ: deltaを加算
                        st.timestamp += ext;
                    }
                    // 継続チャンクの場合はtimestamp変更なし
                } else if st.bytes_left == 0 {
                    // 新しいメッセージ: last_deltaを加算
                    st.timestamp += st.last_delta;
                }
                if st.bytes_left == 0 {
                    st.buf.clear();
                    st.bytes_left = st.msg_length;
                }
            }
            _ => unreachable!(),
        }

        // Read chunk data
        let to_read = std::cmp::min(st.bytes_left, chunk_size) as usize;
        if to_read > 0 {
            let start = st.buf.len();
            st.buf.resize(start + to_read, 0);
            r.read_exact(&mut st.buf[start..]).await
                .map_err(|e| format!("chunk data: {}", e))?;
            total_recv += to_read as u64;
            st.bytes_left -= to_read as u32;
        }

        // Send acknowledgement periodically
        if total_recv - last_ack >= ack_window as u64 {
            last_ack = total_recv;
            send_proto(&mut w, 3, &(total_recv as u32).to_be_bytes()).await?;
        }

        // Message not yet complete
        if st.bytes_left > 0 { continue; }

        let msg_type = st.msg_type_id;
        let timestamp = st.timestamp;
        let msg_data = st.buf.split().freeze();

        match msg_type {
            // Set Chunk Size
            1 if msg_data.len() >= 4 => {
                chunk_size = u32::from_be_bytes([msg_data[0], msg_data[1], msg_data[2], msg_data[3]]) & 0x7FFF_FFFF;
                info!("[NativeRTMP] Peer chunk size: {}", chunk_size);
            }
            // Acknowledgement / Abort / User Control
            2 | 3 | 4 => {}
            // Window Ack Size
            5 if msg_data.len() >= 4 => {
                ack_window = u32::from_be_bytes([msg_data[0], msg_data[1], msg_data[2], msg_data[3]]);
            }
            // Set Peer Bandwidth
            6 => {}
            // Audio (8), Video (9), Data/Metadata (18)
            8 | 9 | 18 => {
                if !flv_started {
                    let hdr = flv_file_header();
                    stream_manager.append_data(&stream_id, &hdr).await;
                    let _ = sender.send(hdr);
                    flv_started = true;
                    info!("[NativeRTMP] FLV started after {:.2}s", t0.elapsed().as_secs_f64());
                }
                let tag = flv_tag(msg_type, timestamp, &msg_data);

                // ヘッダー収集中のみappend_data（ロック取得）
                // 完了後はbroadcastのみ（ゼロロック高速パス）
                if !headers_done {
                    stream_manager.append_data(&stream_id, &tag).await;
                    if stream_manager.is_header_full(&stream_id).await {
                        headers_done = true;
                        info!("[NativeRTMP] Header buffer full, switching to zero-lock fast path");
                    }
                }

                // GOPバッファ管理: 映像キーフレームでリセット
                if msg_type == 9 && msg_data.len() > 0 {
                    // H.264: first byte bit4=frame_type, 1=keyframe
                    let is_keyframe = (msg_data[0] >> 4) == 1;
                    if is_keyframe {
                        gop_buf.clear();
                        got_keyframe = true;
                    }
                }
                if got_keyframe {
                    gop_buf.push(tag.clone());
                    // GOPバッファが大きくなりすぎないよう制限（約5秒分@60fps）
                    if gop_buf.len() > 600 {
                        gop_buf.drain(..100);
                    }
                }

                // StreamManagerにGOPバッファを定期更新
                if msg_type == 9 && msg_data.len() > 0 && (msg_data[0] >> 4) == 1 {
                    stream_manager.update_gop(&stream_id, &gop_buf).await;
                }

                let _ = sender.send(tag);
            }
            // AMF0 Command
            20 => {
                handle_command(&msg_data, &mut w).await?;
            }
            // AMF3 Command (skip encoding byte, handle as AMF0)
            17 if msg_data.len() > 1 => {
                handle_command(&msg_data[1..], &mut w).await?;
            }
            _ => {
                debug!("[NativeRTMP] msg type {}, len {}", msg_type, msg_data.len());
            }
        }
    }

    info!("[NativeRTMP] Session ended, total recv: {} bytes", total_recv);
    Ok(())
}

// ───────── RTMP command handling ─────────

async fn handle_command<W: AsyncWriteExt + Unpin>(data: &[u8], w: &mut W) -> Result<(), String> {
    let (name, tx_id) = match parse_cmd_header(data) {
        Some(v) => v,
        None => return Ok(()),
    };
    info!("[NativeRTMP] Command: {} (tx={})", name, tx_id);

    match name.as_str() {
        "connect" => {
            // OBSはconnect応答の一部としてプロトコルメッセージを期待する
            send_proto(w, 5, &2500000u32.to_be_bytes()).await?; // Window Ack Size
            let mut bw = [0u8; 5];
            bw[..4].copy_from_slice(&2500000u32.to_be_bytes());
            bw[4] = 2; // dynamic
            send_proto(w, 6, &bw).await?; // Set Peer Bandwidth
            send_proto(w, 1, &4096u32.to_be_bytes()).await?; // Set Chunk Size
            send_connect_result(w, tx_id).await?;
            w.flush().await.map_err(|e| format!("flush: {}", e))?;
        }
        "createStream" => {
            send_create_stream_result(w, tx_id).await?;
            w.flush().await.map_err(|e| format!("flush: {}", e))?;
        }
        "publish" => {
            // Stream Begin (User Control, event=0, stream_id=1)
            let sb = [0u8, 0, 0u8, 0, 0u8, 1]; // event=StreamBegin(0), stream=1
            send_proto(w, 4, &sb).await?;
            send_on_status(w).await?;
            w.flush().await.map_err(|e| format!("flush: {}", e))?;
            info!("[NativeRTMP] Publish started");
        }
        "releaseStream" | "FCPublish" | "deleteStream" | "FCUnpublish" => {
            send_null_result(w, tx_id).await?;
            w.flush().await.map_err(|e| format!("flush: {}", e))?;
        }
        _ => { debug!("[NativeRTMP] Ignored command: {}", name); }
    }
    Ok(())
}

fn parse_cmd_header(data: &[u8]) -> Option<(String, f64)> {
    let mut p = 0;
    if p >= data.len() || data[p] != 0x02 { return None; }
    p += 1;
    if p + 2 > data.len() { return None; }
    let len = ((data[p] as usize) << 8) | data[p + 1] as usize;
    p += 2;
    if p + len > data.len() { return None; }
    let name = String::from_utf8_lossy(&data[p..p + len]).to_string();
    p += len;
    let tx = if p < data.len() && data[p] == 0x00 {
        p += 1;
        if p + 8 <= data.len() {
            f64::from_be_bytes(data[p..p + 8].try_into().ok()?)
        } else { 0.0 }
    } else { 0.0 };
    Some((name, tx))
}

// ───────── Send helpers ─────────

/// Protocol control message on cs_id=2, stream_id=0
async fn send_proto<W: AsyncWriteExt + Unpin>(w: &mut W, msg_type: u8, data: &[u8]) -> Result<(), String> {
    let len = data.len() as u32;
    let mut b = BytesMut::with_capacity(12 + data.len());
    b.put_u8(0x02); // fmt=0, cs_id=2
    b.put_u8(0); b.put_u8(0); b.put_u8(0); // timestamp=0
    b.put_u8((len >> 16) as u8); b.put_u8((len >> 8) as u8); b.put_u8(len as u8);
    b.put_u8(msg_type);
    b.put_u32_le(0); // stream_id=0
    b.put_slice(data);
    w.write_all(&b).await.map_err(|e| format!("proto: {}", e))
}

/// AMF0 message with chunking (output chunk size = 4096)
async fn send_amf_msg<W: AsyncWriteExt + Unpin>(w: &mut W, cs_id: u8, stream_id: u32, data: &[u8]) -> Result<(), String> {
    let out_chunk = 4096usize;
    let msg_len = data.len() as u32;
    let mut b = BytesMut::with_capacity(12 + data.len() + 8);
    // fmt=0 header
    b.put_u8(cs_id);
    b.put_u8(0); b.put_u8(0); b.put_u8(0);
    b.put_u8((msg_len >> 16) as u8); b.put_u8((msg_len >> 8) as u8); b.put_u8(msg_len as u8);
    b.put_u8(20); // AMF0 command
    b.put_u32_le(stream_id);
    let first = std::cmp::min(out_chunk, data.len());
    b.put_slice(&data[..first]);
    let mut off = first;
    while off < data.len() {
        b.put_u8(0xC0 | cs_id); // fmt=3
        let end = std::cmp::min(off + out_chunk, data.len());
        b.put_slice(&data[off..end]);
        off = end;
    }
    w.write_all(&b).await.map_err(|e| format!("amf: {}", e))
}

async fn send_connect_result<W: AsyncWriteExt + Unpin>(w: &mut W, tx_id: f64) -> Result<(), String> {
    let mut a = Vec::with_capacity(256);
    amf_str(&mut a, "_result");
    amf_num(&mut a, tx_id);
    // Properties
    a.push(0x03);
    amf_prop_str(&mut a, "fmsVer", "FMS/3,0,1,123");
    amf_prop_num(&mut a, "capabilities", 31.0);
    a.extend_from_slice(&[0, 0, 9]);
    // Info
    a.push(0x03);
    amf_prop_str(&mut a, "level", "status");
    amf_prop_str(&mut a, "code", "NetConnection.Connect.Success");
    amf_prop_str(&mut a, "description", "Connection succeeded.");
    amf_prop_num(&mut a, "objectEncoding", 0.0);
    a.extend_from_slice(&[0, 0, 9]);
    send_amf_msg(w, 3, 0, &a).await
}

async fn send_create_stream_result<W: AsyncWriteExt + Unpin>(w: &mut W, tx_id: f64) -> Result<(), String> {
    let mut a = Vec::with_capacity(32);
    amf_str(&mut a, "_result");
    amf_num(&mut a, tx_id);
    a.push(0x05); // null
    amf_num(&mut a, 1.0); // stream id = 1
    send_amf_msg(w, 3, 0, &a).await
}

async fn send_on_status<W: AsyncWriteExt + Unpin>(w: &mut W) -> Result<(), String> {
    let mut a = Vec::with_capacity(128);
    amf_str(&mut a, "onStatus");
    amf_num(&mut a, 0.0);
    a.push(0x05); // null
    a.push(0x03); // object
    amf_prop_str(&mut a, "level", "status");
    amf_prop_str(&mut a, "code", "NetStream.Publish.Start");
    amf_prop_str(&mut a, "description", "Start publishing");
    a.extend_from_slice(&[0, 0, 9]);
    send_amf_msg(w, 5, 1, &a).await
}

async fn send_null_result<W: AsyncWriteExt + Unpin>(w: &mut W, tx_id: f64) -> Result<(), String> {
    let mut a = Vec::with_capacity(32);
    amf_str(&mut a, "_result");
    amf_num(&mut a, tx_id);
    a.push(0x05); // null
    send_amf_msg(w, 3, 0, &a).await
}

// ───────── AMF0 encoding ─────────

fn amf_str(buf: &mut Vec<u8>, s: &str) {
    buf.push(0x02);
    buf.push((s.len() >> 8) as u8);
    buf.push(s.len() as u8);
    buf.extend_from_slice(s.as_bytes());
}

fn amf_num(buf: &mut Vec<u8>, v: f64) {
    buf.push(0x00);
    buf.extend_from_slice(&v.to_be_bytes());
}

fn amf_prop_str(buf: &mut Vec<u8>, key: &str, val: &str) {
    buf.push((key.len() >> 8) as u8);
    buf.push(key.len() as u8);
    buf.extend_from_slice(key.as_bytes());
    amf_str(buf, val);
}

fn amf_prop_num(buf: &mut Vec<u8>, key: &str, val: f64) {
    buf.push((key.len() >> 8) as u8);
    buf.push(key.len() as u8);
    buf.extend_from_slice(key.as_bytes());
    amf_num(buf, val);
}

// ───────── Byte helpers ─────────

fn read_u24_be(b: &[u8]) -> u32 {
    (b[0] as u32) << 16 | (b[1] as u32) << 8 | b[2] as u32
}

async fn read_u32_be_async<R: AsyncReadExt + Unpin>(r: &mut R) -> Result<u32, String> {
    let mut b = [0u8; 4];
    r.read_exact(&mut b).await.map_err(|e| format!("u32: {}", e))?;
    Ok(u32::from_be_bytes(b))
}
