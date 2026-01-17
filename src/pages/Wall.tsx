import { useState } from "react";
import { useIdentityStore } from "../stores";
import {
  WallIcon,
  PlusIcon,
  EllipsisIcon,
} from "../components/icons";

// Mock data for demonstration
const mockPosts = [
  {
    id: "1",
    content: "Just launched Harbor - a decentralized P2P chat application! It's been an incredible journey building this. Check out the features: end-to-end encryption, local-first data, and peer-to-peer communication. No central servers, no data harvesting. Your identity stays with you.",
    timestamp: new Date(Date.now() - 1000 * 60 * 60 * 2),
    likes: 12,
    comments: 3,
  },
  {
    id: "2",
    content: "The beauty of decentralized systems is that you own your data. No company can access your messages, no algorithm decides what you see. Just direct, secure communication with the people you choose.",
    timestamp: new Date(Date.now() - 1000 * 60 * 60 * 24),
    likes: 8,
    comments: 2,
  },
  {
    id: "3",
    content: "Working on voice calling next! WebRTC signaling through libp2p is going to be interesting. Stay tuned for updates.",
    timestamp: new Date(Date.now() - 1000 * 60 * 60 * 48),
    likes: 15,
    comments: 5,
  },
];

export function WallPage() {
  const { state } = useIdentityStore();
  const [newPost, setNewPost] = useState("");
  const [isComposing, setIsComposing] = useState(false);

  const identity = state.status === "unlocked" ? state.identity : null;

  const formatDate = (date: Date) => {
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const hours = Math.floor(diff / 3600000);
    const days = Math.floor(diff / 86400000);

    if (hours < 1) return "Just now";
    if (hours < 24) return `${hours}h ago`;
    if (days < 7) return `${days}d ago`;
    return date.toLocaleDateString();
  };

  const getInitials = (name: string) => {
    return name
      .split(" ")
      .map((n) => n[0])
      .join("")
      .toUpperCase()
      .slice(0, 2);
  };

  return (
    <div
      className="h-full flex flex-col"
      style={{ background: "hsl(var(--harbor-bg-primary))" }}
    >
      {/* Header */}
      <header
        className="px-6 py-4 border-b flex-shrink-0"
        style={{ borderColor: "hsl(var(--harbor-border-subtle))" }}
      >
        <div className="max-w-2xl mx-auto flex items-center justify-between">
          <div>
            <h1
              className="text-xl font-bold"
              style={{ color: "hsl(var(--harbor-text-primary))" }}
            >
              My Wall
            </h1>
            <p
              className="text-sm mt-0.5"
              style={{ color: "hsl(var(--harbor-text-secondary))" }}
            >
              Share your thoughts with your contacts
            </p>
          </div>
        </div>
      </header>

      <div className="flex-1 overflow-y-auto p-6">
        <div className="max-w-2xl mx-auto space-y-6">
          {/* Composer */}
          <div
            className="rounded-2xl p-4"
            style={{
              background: "hsl(var(--harbor-bg-elevated))",
              border: "1px solid hsl(var(--harbor-border-subtle))",
            }}
          >
            <div className="flex gap-4">
              {/* Avatar */}
              {identity && (
                <div
                  className="w-12 h-12 rounded-full flex items-center justify-center text-sm font-semibold text-white flex-shrink-0"
                  style={{
                    background: "linear-gradient(135deg, hsl(var(--harbor-primary)), hsl(var(--harbor-accent)))",
                  }}
                >
                  {getInitials(identity.displayName)}
                </div>
              )}

              <div className="flex-1">
                <textarea
                  placeholder="What's on your mind?"
                  value={newPost}
                  onChange={(e) => {
                    setNewPost(e.target.value);
                    setIsComposing(true);
                  }}
                  onFocus={() => setIsComposing(true)}
                  rows={isComposing ? 4 : 2}
                  className="w-full resize-none rounded-xl p-3 text-sm transition-all duration-200"
                  style={{
                    background: "hsl(var(--harbor-surface-1))",
                    border: "1px solid hsl(var(--harbor-border-subtle))",
                    color: "hsl(var(--harbor-text-primary))",
                  }}
                />

                {isComposing && (
                  <div className="flex items-center justify-between mt-3">
                    <div className="flex items-center gap-2">
                      <button
                        className="p-2 rounded-lg transition-colors duration-200"
                        style={{
                          background: "hsl(var(--harbor-surface-1))",
                          color: "hsl(var(--harbor-text-secondary))",
                        }}
                        title="Add image"
                      >
                        <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
                        </svg>
                      </button>
                      <button
                        className="p-2 rounded-lg transition-colors duration-200"
                        style={{
                          background: "hsl(var(--harbor-surface-1))",
                          color: "hsl(var(--harbor-text-secondary))",
                        }}
                        title="Add video"
                      >
                        <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z" />
                        </svg>
                      </button>
                    </div>

                    <div className="flex items-center gap-2">
                      <button
                        onClick={() => {
                          setIsComposing(false);
                          setNewPost("");
                        }}
                        className="px-4 py-2 rounded-lg text-sm font-medium transition-colors duration-200"
                        style={{
                          color: "hsl(var(--harbor-text-secondary))",
                        }}
                      >
                        Cancel
                      </button>
                      <button
                        disabled={!newPost.trim()}
                        className="px-4 py-2 rounded-xl text-sm font-medium transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed"
                        style={{
                          background: "linear-gradient(135deg, hsl(var(--harbor-primary)), hsl(var(--harbor-accent)))",
                          color: "white",
                          boxShadow: newPost.trim()
                            ? "0 4px 12px hsl(var(--harbor-primary) / 0.3)"
                            : "none",
                        }}
                      >
                        Post
                      </button>
                    </div>
                  </div>
                )}
              </div>
            </div>
          </div>

          {/* Posts */}
          {mockPosts.length === 0 ? (
            <div className="text-center py-16">
              <div
                className="w-20 h-20 rounded-full flex items-center justify-center mx-auto mb-4"
                style={{ background: "hsl(var(--harbor-surface-1))" }}
              >
                <WallIcon
                  className="w-10 h-10"
                  style={{ color: "hsl(var(--harbor-text-tertiary))" }}
                />
              </div>
              <h3
                className="text-lg font-semibold mb-2"
                style={{ color: "hsl(var(--harbor-text-primary))" }}
              >
                No posts yet
              </h3>
              <p
                className="text-sm max-w-xs mx-auto"
                style={{ color: "hsl(var(--harbor-text-tertiary))" }}
              >
                Share your first post with your contacts. Your posts are stored locally and shared peer-to-peer.
              </p>
            </div>
          ) : (
            mockPosts.map((post) => (
              <article
                key={post.id}
                className="rounded-2xl p-5"
                style={{
                  background: "hsl(var(--harbor-bg-elevated))",
                  border: "1px solid hsl(var(--harbor-border-subtle))",
                }}
              >
                {/* Post header */}
                <div className="flex items-center justify-between mb-4">
                  <div className="flex items-center gap-3">
                    {identity && (
                      <div
                        className="w-10 h-10 rounded-full flex items-center justify-center text-sm font-semibold text-white"
                        style={{
                          background: "linear-gradient(135deg, hsl(var(--harbor-primary)), hsl(var(--harbor-accent)))",
                        }}
                      >
                        {getInitials(identity.displayName)}
                      </div>
                    )}
                    <div>
                      <p
                        className="font-semibold text-sm"
                        style={{ color: "hsl(var(--harbor-text-primary))" }}
                      >
                        {identity?.displayName || "You"}
                      </p>
                      <p
                        className="text-xs"
                        style={{ color: "hsl(var(--harbor-text-tertiary))" }}
                      >
                        {formatDate(post.timestamp)}
                      </p>
                    </div>
                  </div>

                  <button
                    className="p-2 rounded-lg transition-colors duration-200"
                    style={{
                      color: "hsl(var(--harbor-text-tertiary))",
                    }}
                  >
                    <EllipsisIcon className="w-5 h-5" />
                  </button>
                </div>

                {/* Post content */}
                <p
                  className="text-sm leading-relaxed mb-4"
                  style={{ color: "hsl(var(--harbor-text-primary))" }}
                >
                  {post.content}
                </p>

                {/* Post actions */}
                <div
                  className="flex items-center gap-4 pt-4 border-t"
                  style={{ borderColor: "hsl(var(--harbor-border-subtle))" }}
                >
                  <button
                    className="flex items-center gap-2 px-3 py-1.5 rounded-lg transition-colors duration-200"
                    style={{
                      color: "hsl(var(--harbor-text-secondary))",
                    }}
                  >
                    <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z" />
                    </svg>
                    <span className="text-sm">{post.likes}</span>
                  </button>

                  <button
                    className="flex items-center gap-2 px-3 py-1.5 rounded-lg transition-colors duration-200"
                    style={{
                      color: "hsl(var(--harbor-text-secondary))",
                    }}
                  >
                    <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
                    </svg>
                    <span className="text-sm">{post.comments}</span>
                  </button>

                  <button
                    className="flex items-center gap-2 px-3 py-1.5 rounded-lg transition-colors duration-200 ml-auto"
                    style={{
                      color: "hsl(var(--harbor-text-secondary))",
                    }}
                  >
                    <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M8.684 13.342C8.886 12.938 9 12.482 9 12c0-.482-.114-.938-.316-1.342m0 2.684a3 3 0 110-2.684m0 2.684l6.632 3.316m-6.632-6l6.632-3.316m0 0a3 3 0 105.367-2.684 3 3 0 00-5.367 2.684zm0 9.316a3 3 0 105.368 2.684 3 3 0 00-5.368-2.684z" />
                    </svg>
                    <span className="text-sm">Share</span>
                  </button>
                </div>
              </article>
            ))
          )}
        </div>
      </div>
    </div>
  );
}
