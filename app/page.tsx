"use client";

import { StudioLayout } from "@/components/StudioLayout";
import { ChatOverlay } from "@/components/ChatOverlay";
import { ControlBar } from "@/components/ControlBar";
import { VideoPreview } from "@/components/VideoPreview";

export default function Home() {
  return (
    <StudioLayout
      rightPanel={<ChatOverlay />}
      controlBar={<ControlBar />}
    >
      <VideoPreview />
    </StudioLayout>
  );
}
