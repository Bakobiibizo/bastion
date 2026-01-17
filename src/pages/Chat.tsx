import { useState } from "react";
import {
  ChatIcon,
  SearchIcon,
  PlusIcon,
  SendIcon,
  PhoneIcon,
  EllipsisIcon,
} from "../components/icons";

// Mock data for demonstration
const mockConversations = [
  {
    id: "1",
    name: "Alice Chen",
    lastMessage: "Hey! Are you coming to the meetup?",
    timestamp: new Date(Date.now() - 1000 * 60 * 5), // 5 mins ago
    unread: 2,
    online: true,
  },
  {
    id: "2",
    name: "Bob Wilson",
    lastMessage: "Thanks for the help yesterday!",
    timestamp: new Date(Date.now() - 1000 * 60 * 60), // 1 hour ago
    unread: 0,
    online: true,
  },
  {
    id: "3",
    name: "Carol Davis",
    lastMessage: "I'll send over the files soon",
    timestamp: new Date(Date.now() - 1000 * 60 * 60 * 3), // 3 hours ago
    unread: 0,
    online: false,
  },
];

const mockMessages = [
  {
    id: "1",
    senderId: "alice",
    content: "Hey there! How are you?",
    timestamp: new Date(Date.now() - 1000 * 60 * 30),
    isMine: false,
  },
  {
    id: "2",
    senderId: "me",
    content: "I'm doing great! Just working on the Harbor app",
    timestamp: new Date(Date.now() - 1000 * 60 * 28),
    isMine: true,
  },
  {
    id: "3",
    senderId: "alice",
    content: "That sounds exciting! How is it going?",
    timestamp: new Date(Date.now() - 1000 * 60 * 25),
    isMine: false,
  },
  {
    id: "4",
    senderId: "me",
    content: "Really well! The P2P networking is working beautifully",
    timestamp: new Date(Date.now() - 1000 * 60 * 20),
    isMine: true,
  },
  {
    id: "5",
    senderId: "alice",
    content: "Are you coming to the meetup?",
    timestamp: new Date(Date.now() - 1000 * 60 * 5),
    isMine: false,
  },
];

export function ChatPage() {
  const [selectedConversation, setSelectedConversation] = useState<string | null>("1");
  const [searchQuery, setSearchQuery] = useState("");
  const [messageInput, setMessageInput] = useState("");

  const formatTime = (date: Date) => {
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const mins = Math.floor(diff / 60000);
    const hours = Math.floor(diff / 3600000);
    const days = Math.floor(diff / 86400000);

    if (mins < 1) return "now";
    if (mins < 60) return `${mins}m`;
    if (hours < 24) return `${hours}h`;
    return `${days}d`;
  };

  const getInitials = (name: string) => {
    return name
      .split(" ")
      .map((n) => n[0])
      .join("")
      .toUpperCase()
      .slice(0, 2);
  };

  const filteredConversations = mockConversations.filter((c) =>
    c.name.toLowerCase().includes(searchQuery.toLowerCase())
  );

  const selectedConv = mockConversations.find((c) => c.id === selectedConversation);

  return (
    <div
      className="h-full flex"
      style={{ background: "hsl(var(--harbor-bg-primary))" }}
    >
      {/* Conversations sidebar */}
      <div
        className="w-80 flex flex-col border-r flex-shrink-0"
        style={{
          borderColor: "hsl(var(--harbor-border-subtle))",
          background: "hsl(var(--harbor-bg-elevated))",
        }}
      >
        {/* Header */}
        <div
          className="p-4 border-b"
          style={{ borderColor: "hsl(var(--harbor-border-subtle))" }}
        >
          <div className="flex items-center justify-between mb-4">
            <h2
              className="text-lg font-bold"
              style={{ color: "hsl(var(--harbor-text-primary))" }}
            >
              Messages
            </h2>
            <button
              className="p-2 rounded-lg transition-colors duration-200"
              style={{
                background: "linear-gradient(135deg, hsl(var(--harbor-primary)), hsl(var(--harbor-accent)))",
                color: "white",
              }}
            >
              <PlusIcon className="w-4 h-4" />
            </button>
          </div>

          {/* Search */}
          <div className="relative">
            <SearchIcon
              className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4"
              style={{ color: "hsl(var(--harbor-text-tertiary))" }}
            />
            <input
              type="text"
              placeholder="Search conversations..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full pl-10 pr-4 py-2 rounded-lg text-sm"
              style={{
                background: "hsl(var(--harbor-surface-1))",
                border: "1px solid hsl(var(--harbor-border-subtle))",
                color: "hsl(var(--harbor-text-primary))",
              }}
            />
          </div>
        </div>

        {/* Conversation list */}
        <div className="flex-1 overflow-y-auto p-2">
          {filteredConversations.length === 0 ? (
            <div className="text-center py-8">
              <ChatIcon
                className="w-12 h-12 mx-auto mb-3"
                style={{ color: "hsl(var(--harbor-text-tertiary))" }}
              />
              <p
                className="text-sm"
                style={{ color: "hsl(var(--harbor-text-tertiary))" }}
              >
                No conversations found
              </p>
            </div>
          ) : (
            <div className="space-y-1">
              {filteredConversations.map((conversation) => (
                <button
                  key={conversation.id}
                  onClick={() => setSelectedConversation(conversation.id)}
                  className="w-full flex items-center gap-3 p-3 rounded-xl text-left transition-all duration-200"
                  style={{
                    background:
                      selectedConversation === conversation.id
                        ? "linear-gradient(135deg, hsl(var(--harbor-primary) / 0.15), hsl(var(--harbor-accent) / 0.1))"
                        : "transparent",
                    border:
                      selectedConversation === conversation.id
                        ? "1px solid hsl(var(--harbor-primary) / 0.2)"
                        : "1px solid transparent",
                  }}
                >
                  {/* Avatar */}
                  <div className="relative flex-shrink-0">
                    <div
                      className="w-12 h-12 rounded-full flex items-center justify-center text-sm font-semibold text-white"
                      style={{
                        background: "linear-gradient(135deg, hsl(var(--harbor-primary)), hsl(var(--harbor-accent)))",
                      }}
                    >
                      {getInitials(conversation.name)}
                    </div>
                    {conversation.online && (
                      <div
                        className="absolute -bottom-0.5 -right-0.5 w-3.5 h-3.5 rounded-full border-2"
                        style={{
                          background: "hsl(var(--harbor-success))",
                          borderColor: "hsl(var(--harbor-bg-elevated))",
                        }}
                      />
                    )}
                  </div>

                  {/* Info */}
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center justify-between mb-0.5">
                      <p
                        className="font-semibold text-sm truncate"
                        style={{ color: "hsl(var(--harbor-text-primary))" }}
                      >
                        {conversation.name}
                      </p>
                      <span
                        className="text-xs flex-shrink-0 ml-2"
                        style={{ color: "hsl(var(--harbor-text-tertiary))" }}
                      >
                        {formatTime(conversation.timestamp)}
                      </span>
                    </div>
                    <div className="flex items-center justify-between">
                      <p
                        className="text-sm truncate"
                        style={{ color: "hsl(var(--harbor-text-secondary))" }}
                      >
                        {conversation.lastMessage}
                      </p>
                      {conversation.unread > 0 && (
                        <span
                          className="ml-2 px-1.5 py-0.5 rounded-full text-xs font-semibold flex-shrink-0"
                          style={{
                            background: "linear-gradient(135deg, hsl(var(--harbor-primary)), hsl(var(--harbor-accent)))",
                            color: "white",
                          }}
                        >
                          {conversation.unread}
                        </span>
                      )}
                    </div>
                  </div>
                </button>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Chat area */}
      {selectedConversation && selectedConv ? (
        <div className="flex-1 flex flex-col">
          {/* Chat header */}
          <header
            className="px-6 py-4 border-b flex items-center justify-between flex-shrink-0"
            style={{
              borderColor: "hsl(var(--harbor-border-subtle))",
              background: "hsl(var(--harbor-bg-elevated))",
            }}
          >
            <div className="flex items-center gap-3">
              <div className="relative">
                <div
                  className="w-10 h-10 rounded-full flex items-center justify-center text-sm font-semibold text-white"
                  style={{
                    background: "linear-gradient(135deg, hsl(var(--harbor-primary)), hsl(var(--harbor-accent)))",
                  }}
                >
                  {getInitials(selectedConv.name)}
                </div>
                {selectedConv.online && (
                  <div
                    className="absolute -bottom-0.5 -right-0.5 w-3 h-3 rounded-full border-2"
                    style={{
                      background: "hsl(var(--harbor-success))",
                      borderColor: "hsl(var(--harbor-bg-elevated))",
                    }}
                  />
                )}
              </div>
              <div>
                <p
                  className="font-semibold"
                  style={{ color: "hsl(var(--harbor-text-primary))" }}
                >
                  {selectedConv.name}
                </p>
                <p
                  className="text-xs"
                  style={{
                    color: selectedConv.online
                      ? "hsl(var(--harbor-success))"
                      : "hsl(var(--harbor-text-tertiary))",
                  }}
                >
                  {selectedConv.online ? "Online" : "Offline"}
                </p>
              </div>
            </div>

            <div className="flex items-center gap-2">
              <button
                className="p-2 rounded-lg transition-colors duration-200"
                style={{
                  background: "hsl(var(--harbor-success) / 0.15)",
                  color: "hsl(var(--harbor-success))",
                }}
              >
                <PhoneIcon className="w-5 h-5" />
              </button>
              <button
                className="p-2 rounded-lg transition-colors duration-200"
                style={{
                  background: "hsl(var(--harbor-surface-1))",
                  color: "hsl(var(--harbor-text-secondary))",
                }}
              >
                <EllipsisIcon className="w-5 h-5" />
              </button>
            </div>
          </header>

          {/* Messages */}
          <div className="flex-1 overflow-y-auto p-6">
            <div className="max-w-3xl mx-auto space-y-4">
              {mockMessages.map((message) => (
                <div
                  key={message.id}
                  className={`flex ${message.isMine ? "justify-end" : "justify-start"}`}
                >
                  <div
                    className="max-w-[70%] px-4 py-3 rounded-2xl"
                    style={{
                      background: message.isMine
                        ? "linear-gradient(135deg, hsl(var(--harbor-primary)), hsl(var(--harbor-accent)))"
                        : "hsl(var(--harbor-surface-1))",
                      color: message.isMine ? "white" : "hsl(var(--harbor-text-primary))",
                      borderBottomRightRadius: message.isMine ? "4px" : "16px",
                      borderBottomLeftRadius: message.isMine ? "16px" : "4px",
                    }}
                  >
                    <p className="text-sm">{message.content}</p>
                    <p
                      className="text-xs mt-1 text-right"
                      style={{
                        color: message.isMine
                          ? "rgba(255,255,255,0.7)"
                          : "hsl(var(--harbor-text-tertiary))",
                      }}
                    >
                      {message.timestamp.toLocaleTimeString([], {
                        hour: "2-digit",
                        minute: "2-digit",
                      })}
                    </p>
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* Message input */}
          <div
            className="p-4 border-t"
            style={{
              borderColor: "hsl(var(--harbor-border-subtle))",
              background: "hsl(var(--harbor-bg-elevated))",
            }}
          >
            <div className="max-w-3xl mx-auto flex items-center gap-3">
              <input
                type="text"
                placeholder="Type a message..."
                value={messageInput}
                onChange={(e) => setMessageInput(e.target.value)}
                className="flex-1 px-4 py-3 rounded-xl text-sm"
                style={{
                  background: "hsl(var(--harbor-surface-1))",
                  border: "1px solid hsl(var(--harbor-border-subtle))",
                  color: "hsl(var(--harbor-text-primary))",
                }}
              />
              <button
                className="p-3 rounded-xl transition-all duration-200"
                style={{
                  background: messageInput.trim()
                    ? "linear-gradient(135deg, hsl(var(--harbor-primary)), hsl(var(--harbor-accent)))"
                    : "hsl(var(--harbor-surface-2))",
                  color: messageInput.trim() ? "white" : "hsl(var(--harbor-text-tertiary))",
                  boxShadow: messageInput.trim()
                    ? "0 4px 12px hsl(var(--harbor-primary) / 0.3)"
                    : "none",
                }}
              >
                <SendIcon className="w-5 h-5" />
              </button>
            </div>
          </div>
        </div>
      ) : (
        /* Empty state */
        <div className="flex-1 flex items-center justify-center">
          <div className="text-center">
            <div
              className="w-20 h-20 rounded-full flex items-center justify-center mx-auto mb-4"
              style={{ background: "hsl(var(--harbor-surface-1))" }}
            >
              <ChatIcon
                className="w-10 h-10"
                style={{ color: "hsl(var(--harbor-text-tertiary))" }}
              />
            </div>
            <h3
              className="text-lg font-semibold mb-2"
              style={{ color: "hsl(var(--harbor-text-primary))" }}
            >
              Select a conversation
            </h3>
            <p
              className="text-sm max-w-xs"
              style={{ color: "hsl(var(--harbor-text-tertiary))" }}
            >
              Choose a conversation from the sidebar or start a new one to begin messaging
            </p>
          </div>
        </div>
      )}
    </div>
  );
}
