"use client";

import React, { useState } from 'react';
import { StudioLayout } from "@/components/StudioLayout";
import { ChatOverlay } from "@/components/ChatOverlay";
import { ControlBar } from "@/components/ControlBar";
import { VideoPreview } from "@/components/VideoPreview";

export default function Home() {
  const [isStreaming, setIsStreaming] = useState(false);

  return (
    <StudioLayout
      rightPanel={<ChatOverlay isStreaming={isStreaming} />}
      controlBar={<ControlBar isStreaming={isStreaming} onToggleStreaming={setIsStreaming} />}
    >
      <VideoPreview />
    </StudioLayout>
  );
}
