import { FeedIcon, EllipsisIcon } from "../components/icons";

// Mock data for demonstration
const mockFeedPosts = [
  {
    id: "1",
    author: {
      name: "Alice Chen",
      peerId: "12D3KooWAbCdEfGhIjKlMnOpQrStUvWxYz",
    },
    content: "Just finished reading 'The Network State' by Balaji. Fascinating ideas about how digital communities can evolve into something more. What are your thoughts on decentralized governance?",
    timestamp: new Date(Date.now() - 1000 * 60 * 30),
    likes: 24,
    comments: 8,
  },
  {
    id: "2",
    author: {
      name: "Bob Wilson",
      peerId: "12D3KooWXyZaBcDeFgHiJkLmNoPqRsTuVw",
    },
    content: "Excited to announce that I'm joining the Harbor project as a contributor! Looking forward to building the future of decentralized communication together.",
    timestamp: new Date(Date.now() - 1000 * 60 * 60 * 4),
    likes: 45,
    comments: 12,
  },
  {
    id: "3",
    author: {
      name: "Carol Davis",
      peerId: "12D3KooWQrStUvWxYzAbCdEfGhIjKlMnOp",
    },
    content: "Tip for fellow developers: When working with libp2p, make sure to handle peer disconnections gracefully. The network is inherently unstable, and your app needs to handle that elegantly.",
    timestamp: new Date(Date.now() - 1000 * 60 * 60 * 12),
    likes: 18,
    comments: 5,
  },
  {
    id: "4",
    author: {
      name: "David Miller",
      peerId: "12D3KooWMnOpQrStUvWxYzAbCdEfGhIjKl",
    },
    content: "The more I use peer-to-peer apps, the more I realize how much we've given up to centralized platforms. Privacy isn't just a feature—it's a right.",
    timestamp: new Date(Date.now() - 1000 * 60 * 60 * 24),
    likes: 67,
    comments: 21,
  },
];

export function FeedPage() {
  const formatDate = (date: Date) => {
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const mins = Math.floor(diff / 60000);
    const hours = Math.floor(diff / 3600000);
    const days = Math.floor(diff / 86400000);

    if (mins < 1) return "Just now";
    if (mins < 60) return `${mins}m ago`;
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

  // Generate consistent color for each peer
  const getAvatarGradient = (peerId: string) => {
    const colors = [
      "linear-gradient(135deg, hsl(220 91% 54%), hsl(262 83% 58%))", // primary/accent
      "linear-gradient(135deg, hsl(262 83% 58%), hsl(330 81% 60%))", // purple/pink
      "linear-gradient(135deg, hsl(152 69% 40%), hsl(180 70% 45%))", // green/teal
      "linear-gradient(135deg, hsl(36 90% 55%), hsl(15 80% 55%))", // orange/red
      "linear-gradient(135deg, hsl(200 80% 50%), hsl(220 91% 54%))", // blue/primary
    ];
    const index = peerId.charCodeAt(0) % colors.length;
    return colors[index];
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
              Feed
            </h1>
            <p
              className="text-sm mt-0.5"
              style={{ color: "hsl(var(--harbor-text-secondary))" }}
            >
              Updates from your contacts
            </p>
          </div>

          <button
            className="px-4 py-2 rounded-xl text-sm font-medium transition-all duration-200"
            style={{
              background: "hsl(var(--harbor-surface-1))",
              color: "hsl(var(--harbor-text-secondary))",
              border: "1px solid hsl(var(--harbor-border-subtle))",
            }}
          >
            Refresh
          </button>
        </div>
      </header>

      <div className="flex-1 overflow-y-auto p-6">
        <div className="max-w-2xl mx-auto space-y-6">
          {mockFeedPosts.length === 0 ? (
            <div className="text-center py-16">
              <div
                className="w-20 h-20 rounded-full flex items-center justify-center mx-auto mb-4"
                style={{ background: "hsl(var(--harbor-surface-1))" }}
              >
                <FeedIcon
                  className="w-10 h-10"
                  style={{ color: "hsl(var(--harbor-text-tertiary))" }}
                />
              </div>
              <h3
                className="text-lg font-semibold mb-2"
                style={{ color: "hsl(var(--harbor-text-primary))" }}
              >
                Your feed is empty
              </h3>
              <p
                className="text-sm max-w-xs mx-auto mb-4"
                style={{ color: "hsl(var(--harbor-text-tertiary))" }}
              >
                When your contacts share posts and grant you permission to view them, they'll appear here.
              </p>
              <button
                className="px-4 py-2 rounded-xl text-sm font-medium transition-all duration-200"
                style={{
                  background: "linear-gradient(135deg, hsl(var(--harbor-primary)), hsl(var(--harbor-accent)))",
                  color: "white",
                  boxShadow: "0 4px 12px hsl(var(--harbor-primary) / 0.3)",
                }}
              >
                Find Contacts
              </button>
            </div>
          ) : (
            mockFeedPosts.map((post) => (
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
                    <div
                      className="w-10 h-10 rounded-full flex items-center justify-center text-sm font-semibold text-white"
                      style={{
                        background: getAvatarGradient(post.author.peerId),
                      }}
                    >
                      {getInitials(post.author.name)}
                    </div>
                    <div>
                      <p
                        className="font-semibold text-sm"
                        style={{ color: "hsl(var(--harbor-text-primary))" }}
                      >
                        {post.author.name}
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
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z" />
                    </svg>
                    <span className="text-sm">Save</span>
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
