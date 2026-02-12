
export function request_fullscreen(id) {
    const el = document.getElementById(id);
    if (!el) return;
    if (el.requestFullscreen) el.requestFullscreen();
    else if (el.webkitRequestFullscreen) el.webkitRequestFullscreen();
}
export function toggle_video_mute(id) {
    const el = document.getElementById(id);
    if (el) el.muted = !el.muted;
    return el ? el.muted : true;
}
export function toggle_video_play(id) {
    const el = document.getElementById(id);
    if (!el) return true;
    if (el.paused) { el.play(); return false; }
    else { el.pause(); return true; }
}
export function set_video_volume(id, vol) {
    const el = document.getElementById(id);
    if (el) { el.volume = vol; el.muted = false; }
}
export function copy_to_clipboard(text) {
    navigator.clipboard.writeText(text);
}
