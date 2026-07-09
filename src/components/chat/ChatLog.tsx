import { useState, useEffect, useRef } from "react";
import { useBot, useBotEvents, useMemory, HistoryEntry } from "../../hooks/useTauri";

interface ChatMessage {
  id: number;
  timestamp: string;
  player: string;
  message: string;
  isBot: boolean;
  isSystem: boolean;
}

export default function ChatLog() {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [input, setInput] = useState("");
  const chatEndRef = useRef<HTMLDivElement>(null);
  const { sendChat } = useBot();
  const { getHistory } = useMemory();

  useEffect(() => {
    const loadHistory = async () => {
      try {
        const history = await getHistory(50);
        const chatMessages: ChatMessage[] = history
          .filter((h) => h.event_type === "chat" || h.event_type === "system")
          .map((h, i) => ({
            id: i,
            timestamp: new Date(h.timestamp).toLocaleTimeString("en-US", { hour12: false, hour: "2-digit", minute: "2-digit" }),
            player: h.player || "System",
            message: h.details || "",
            isBot: h.player === "MineMate",
            isSystem: h.event_type === "system",
          }));
        setMessages(chatMessages);
      } catch (e) {
        console.error("Failed to load chat history:", e);
      }
    };
    loadHistory();
  }, [getHistory]);

  useBotEvents((event) => {
    if (event.type === "ChatMessage") {
      const newMsg: ChatMessage = {
        id: messages.length + 1,
        timestamp: new Date().toLocaleTimeString("en-US", { hour12: false, hour: "2-digit", minute: "2-digit" }),
        player: event.data.player,
        message: event.data.message,
        isBot: event.data.player === "MineMate",
        isSystem: false,
      };
      setMessages((prev) => [...prev, newMsg]);
    }
  });

  useEffect(() => {
    chatEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  const sendMessage = async () => {
    if (!input.trim()) return;

    const newMsg: ChatMessage = {
      id: messages.length + 1,
      timestamp: new Date().toLocaleTimeString("en-US", { hour12: false, hour: "2-digit", minute: "2-digit" }),
      player: "You",
      message: input,
      isBot: false,
      isSystem: false,
    };

    setMessages([...messages, newMsg]);
    setInput("");

    try {
      await sendChat(input);
    } catch (e) {
      console.error("Failed to send chat:", e);
    }
  };

  return (
    <div>
      <h1 className="font-mono" style={{ fontSize: "24px", fontWeight: 700, color: "var(--primary-fixed)", marginBottom: "24px" }}>
        Server Chat
      </h1>

      {/* Chat Messages */}
      <div className="mc-bevel-in" style={{ padding: "16px", height: "60vh", overflowY: "auto", marginBottom: "16px" }}>
        {messages.length === 0 && (
          <div style={{ color: "var(--mc-gray)", fontStyle: "italic" }}>
            No messages yet. Start chatting!
          </div>
        )}
        {messages.map((msg) => (
          <div key={msg.id} style={{ marginBottom: "8px", fontFamily: "'Space Mono', monospace", fontSize: "14px" }}>
            <span style={{ color: "var(--mc-gray)" }}>[{msg.timestamp}] </span>
            {msg.isSystem ? (
              <span style={{ color: "var(--mc-yellow)", fontStyle: "italic" }}>{msg.message}</span>
            ) : msg.isBot ? (
              <>
                <span className="mc-button-primary" style={{ padding: "2px 6px", fontSize: "12px", marginRight: "6px" }}>[BOT]</span>
                <span style={{ color: "var(--mc-green)" }}>{msg.player}: </span>
                <span style={{ color: "var(--on-surface)" }}>{msg.message}</span>
              </>
            ) : (
              <>
                <span style={{ color: "var(--mc-aqua)" }}>&lt;{msg.player}&gt; </span>
                <span style={{ color: "var(--on-surface)" }}>{msg.message}</span>
              </>
            )}
          </div>
        ))}
        <div ref={chatEndRef} />
      </div>

      {/* Chat Input */}
      <div className="mc-bevel-in" style={{ padding: "12px", display: "flex", gap: "8px" }}>
        <span className="font-mono" style={{ color: "var(--mc-green)", lineHeight: "40px" }}>&gt;</span>
        <input
          className="mc-input"
          style={{ flex: 1, height: "40px" }}
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && sendMessage()}
          placeholder="Type a message..."
        />
        <button className="mc-button mc-button-primary" onClick={sendMessage}>
          SEND
        </button>
      </div>

      {/* XP Bar */}
      <div style={{ marginTop: "16px", maxWidth: "600px" }}>
        <div style={{ display: "flex", justifyContent: "space-between", marginBottom: "4px" }}>
          <span className="font-mono" style={{ fontSize: "12px", color: "var(--mc-green)" }}>CHAT LEVEL {messages.length}</span>
          <span className="font-mono" style={{ fontSize: "12px", color: "var(--mc-green)" }}>MESSAGES: {messages.length}</span>
        </div>
        <div className="xp-bar">
          <div className="xp-bar-fill" style={{ width: `${Math.min(messages.length * 2, 100)}%` }} />
        </div>
      </div>
    </div>
  );
}
