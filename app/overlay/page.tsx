import { ChatOverlay } from "@/components/ChatOverlay";

export default function OverlayPage() {
    return (
        <div className="w-full h-screen bg-transparent overflow-hidden">
            <ChatOverlay transparent={true} />
        </div>
    );
}
