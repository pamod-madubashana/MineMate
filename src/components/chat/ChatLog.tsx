import { useState, useEffect, useRef } from "react";

interface ChatMessage {
  id: number;
  timestamp: string;
  player: string;
  message: string;
  isBot: boolean;
  isSystem: boolean;
}

export default function ChatLog() {
  const [messages, setMessages] = useState<ChatMessage[]>([
    { id: 1, timestamp: "14:02", player: "Server", message: "Welcome to MineMate AI!", isBot: false, isSystem: true },
    { id: 2, timestamp: "14:02", player: "MineMate", message: "Bot connected and ready.", isBot: true, isSystem: false },
  ]);
  const [input, setInput] = useState("");
  const chatEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    chatEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  const sendMessage = () => {
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
  };

  return (
    <div>
      <h1 className="font-mono" style={{ fontSize: "24px", fontWeight: 700, color: "var(--primary-fixed)", marginBottom: "24px" }}>
        Server Chat
      </h1>

      {/* Chat Messages */}
      <div className="mc-bevel-in" style={{ padding: "16px", height: "60vh", overflowY: "auto", marginBottom: "16px" }}>
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
          <span className="font-mono" style={{ fontSize: "12px", color: "var(--mc-green)" }}>XP LEVEL 0</span>
          <span className="font-mono" style={{ fontSize: "12px", color: "var(--mc-green)" }}>SERVER LOAD: 0%</span>
        </div>
        <div className="xp-bar">
          <div className="xp-bar-fill" style={{ width: "0%" }} />
        </div>
      </div>
    </div>
  );
}
