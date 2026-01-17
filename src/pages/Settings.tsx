import { useState } from "react";
import { useIdentityStore } from "../stores";
import {
  SettingsIcon,
  UserIcon,
  LockIcon,
  NetworkIcon,
  ShieldIcon,
  ChevronRightIcon,
} from "../components/icons";

export function SettingsPage() {
  const { state } = useIdentityStore();
  const [activeSection, setActiveSection] = useState<string>("profile");

  const identity = state.status === "unlocked" ? state.identity : null;

  const getInitials = (name: string) => {
    return name
      .split(" ")
      .map((n) => n[0])
      .join("")
      .toUpperCase()
      .slice(0, 2);
  };

  const sections = [
    { id: "profile", label: "Profile", icon: UserIcon, description: "Your identity and bio" },
    { id: "security", label: "Security", icon: LockIcon, description: "Passphrase and keys" },
    { id: "network", label: "Network", icon: NetworkIcon, description: "P2P settings" },
    { id: "privacy", label: "Privacy", icon: ShieldIcon, description: "Permissions and visibility" },
  ];

  return (
    <div
      className="h-full flex"
      style={{ background: "hsl(var(--harbor-bg-primary))" }}
    >
      {/* Settings sidebar */}
      <div
        className="w-72 flex flex-col border-r flex-shrink-0"
        style={{
          borderColor: "hsl(var(--harbor-border-subtle))",
          background: "hsl(var(--harbor-bg-elevated))",
        }}
      >
        <div
          className="p-4 border-b"
          style={{ borderColor: "hsl(var(--harbor-border-subtle))" }}
        >
          <h2
            className="text-lg font-bold"
            style={{ color: "hsl(var(--harbor-text-primary))" }}
          >
            Settings
          </h2>
          <p
            className="text-sm mt-0.5"
            style={{ color: "hsl(var(--harbor-text-secondary))" }}
          >
            Customize your experience
          </p>
        </div>

        <nav className="flex-1 p-2 space-y-1">
          {sections.map((section) => {
            const Icon = section.icon;
            const isActive = activeSection === section.id;

            return (
              <button
                key={section.id}
                onClick={() => setActiveSection(section.id)}
                className="w-full flex items-center gap-3 p-3 rounded-xl text-left transition-all duration-200"
                style={{
                  background: isActive
                    ? "linear-gradient(135deg, hsl(var(--harbor-primary) / 0.15), hsl(var(--harbor-accent) / 0.1))"
                    : "transparent",
                  border: isActive
                    ? "1px solid hsl(var(--harbor-primary) / 0.2)"
                    : "1px solid transparent",
                }}
              >
                <div
                  className="w-9 h-9 rounded-lg flex items-center justify-center"
                  style={{
                    background: isActive
                      ? "linear-gradient(135deg, hsl(var(--harbor-primary)), hsl(var(--harbor-accent)))"
                      : "hsl(var(--harbor-surface-2))",
                  }}
                >
                  <Icon
                    className="w-5 h-5"
                    style={{
                      color: isActive ? "white" : "hsl(var(--harbor-text-secondary))",
                    }}
                  />
                </div>
                <div className="flex-1 min-w-0">
                  <p
                    className="font-medium text-sm"
                    style={{
                      color: isActive
                        ? "hsl(var(--harbor-primary))"
                        : "hsl(var(--harbor-text-primary))",
                    }}
                  >
                    {section.label}
                  </p>
                  <p
                    className="text-xs truncate"
                    style={{ color: "hsl(var(--harbor-text-tertiary))" }}
                  >
                    {section.description}
                  </p>
                </div>
                <ChevronRightIcon
                  className="w-4 h-4"
                  style={{
                    color: isActive
                      ? "hsl(var(--harbor-primary))"
                      : "hsl(var(--harbor-text-tertiary))",
                  }}
                />
              </button>
            );
          })}
        </nav>

        {/* Version info */}
        <div
          className="p-4 border-t"
          style={{ borderColor: "hsl(var(--harbor-border-subtle))" }}
        >
          <p
            className="text-xs text-center"
            style={{ color: "hsl(var(--harbor-text-tertiary))" }}
          >
            Harbor v1.0.0
          </p>
        </div>
      </div>

      {/* Settings content */}
      <div className="flex-1 overflow-y-auto p-6">
        <div className="max-w-2xl mx-auto">
          {activeSection === "profile" && (
            <div className="space-y-6">
              <div>
                <h3
                  className="text-lg font-semibold mb-1"
                  style={{ color: "hsl(var(--harbor-text-primary))" }}
                >
                  Profile
                </h3>
                <p
                  className="text-sm"
                  style={{ color: "hsl(var(--harbor-text-secondary))" }}
                >
                  Manage your identity and how others see you
                </p>
              </div>

              {/* Avatar section */}
              <div
                className="rounded-2xl p-6"
                style={{
                  background: "hsl(var(--harbor-bg-elevated))",
                  border: "1px solid hsl(var(--harbor-border-subtle))",
                }}
              >
                <div className="flex items-center gap-6">
                  {identity && (
                    <div
                      className="w-20 h-20 rounded-full flex items-center justify-center text-2xl font-semibold text-white flex-shrink-0"
                      style={{
                        background: "linear-gradient(135deg, hsl(var(--harbor-primary)), hsl(var(--harbor-accent)))",
                      }}
                    >
                      {getInitials(identity.displayName)}
                    </div>
                  )}
                  <div className="flex-1">
                    <h4
                      className="font-medium mb-2"
                      style={{ color: "hsl(var(--harbor-text-primary))" }}
                    >
                      Profile Photo
                    </h4>
                    <p
                      className="text-sm mb-3"
                      style={{ color: "hsl(var(--harbor-text-secondary))" }}
                    >
                      Upload a photo to personalize your profile
                    </p>
                    <button
                      className="px-4 py-2 rounded-lg text-sm font-medium transition-colors duration-200"
                      style={{
                        background: "hsl(var(--harbor-surface-1))",
                        color: "hsl(var(--harbor-text-primary))",
                        border: "1px solid hsl(var(--harbor-border-subtle))",
                      }}
                    >
                      Upload Photo
                    </button>
                  </div>
                </div>
              </div>

              {/* Display name */}
              <div
                className="rounded-2xl p-6"
                style={{
                  background: "hsl(var(--harbor-bg-elevated))",
                  border: "1px solid hsl(var(--harbor-border-subtle))",
                }}
              >
                <label
                  className="block text-sm font-medium mb-2"
                  style={{ color: "hsl(var(--harbor-text-primary))" }}
                >
                  Display Name
                </label>
                <input
                  type="text"
                  defaultValue={identity?.displayName || ""}
                  className="w-full px-4 py-3 rounded-xl text-sm"
                  style={{
                    background: "hsl(var(--harbor-surface-1))",
                    border: "1px solid hsl(var(--harbor-border-subtle))",
                    color: "hsl(var(--harbor-text-primary))",
                  }}
                />
              </div>

              {/* Bio */}
              <div
                className="rounded-2xl p-6"
                style={{
                  background: "hsl(var(--harbor-bg-elevated))",
                  border: "1px solid hsl(var(--harbor-border-subtle))",
                }}
              >
                <label
                  className="block text-sm font-medium mb-2"
                  style={{ color: "hsl(var(--harbor-text-primary))" }}
                >
                  Bio
                </label>
                <textarea
                  defaultValue={identity?.bio || ""}
                  rows={3}
                  placeholder="Tell others about yourself..."
                  className="w-full px-4 py-3 rounded-xl text-sm resize-none"
                  style={{
                    background: "hsl(var(--harbor-surface-1))",
                    border: "1px solid hsl(var(--harbor-border-subtle))",
                    color: "hsl(var(--harbor-text-primary))",
                  }}
                />
              </div>

              {/* Peer ID */}
              <div
                className="rounded-2xl p-6"
                style={{
                  background: "hsl(var(--harbor-bg-elevated))",
                  border: "1px solid hsl(var(--harbor-border-subtle))",
                }}
              >
                <label
                  className="block text-sm font-medium mb-2"
                  style={{ color: "hsl(var(--harbor-text-primary))" }}
                >
                  Peer ID
                </label>
                <div className="flex gap-2">
                  <div
                    className="flex-1 px-4 py-3 rounded-xl text-sm font-mono truncate"
                    style={{
                      background: "hsl(var(--harbor-surface-1))",
                      border: "1px solid hsl(var(--harbor-border-subtle))",
                      color: "hsl(var(--harbor-text-secondary))",
                    }}
                  >
                    {identity?.peerId || "No identity"}
                  </div>
                  <button
                    onClick={() =>
                      identity && navigator.clipboard.writeText(identity.peerId)
                    }
                    className="px-4 py-3 rounded-xl text-sm font-medium transition-colors duration-200"
                    style={{
                      background: "hsl(var(--harbor-surface-1))",
                      color: "hsl(var(--harbor-text-primary))",
                      border: "1px solid hsl(var(--harbor-border-subtle))",
                    }}
                  >
                    Copy
                  </button>
                </div>
                <p
                  className="text-xs mt-2"
                  style={{ color: "hsl(var(--harbor-text-tertiary))" }}
                >
                  Share this ID with others so they can add you as a contact
                </p>
              </div>

              {/* Save button */}
              <div className="flex justify-end">
                <button
                  className="px-6 py-3 rounded-xl text-sm font-medium transition-all duration-200"
                  style={{
                    background: "linear-gradient(135deg, hsl(var(--harbor-primary)), hsl(var(--harbor-accent)))",
                    color: "white",
                    boxShadow: "0 4px 12px hsl(var(--harbor-primary) / 0.3)",
                  }}
                >
                  Save Changes
                </button>
              </div>
            </div>
          )}

          {activeSection === "security" && (
            <div className="space-y-6">
              <div>
                <h3
                  className="text-lg font-semibold mb-1"
                  style={{ color: "hsl(var(--harbor-text-primary))" }}
                >
                  Security
                </h3>
                <p
                  className="text-sm"
                  style={{ color: "hsl(var(--harbor-text-secondary))" }}
                >
                  Manage your passphrase and encryption keys
                </p>
              </div>

              {/* Change passphrase */}
              <div
                className="rounded-2xl p-6"
                style={{
                  background: "hsl(var(--harbor-bg-elevated))",
                  border: "1px solid hsl(var(--harbor-border-subtle))",
                }}
              >
                <h4
                  className="font-medium mb-2"
                  style={{ color: "hsl(var(--harbor-text-primary))" }}
                >
                  Change Passphrase
                </h4>
                <p
                  className="text-sm mb-4"
                  style={{ color: "hsl(var(--harbor-text-secondary))" }}
                >
                  Update your passphrase to keep your identity secure
                </p>

                <div className="space-y-3">
                  <input
                    type="password"
                    placeholder="Current passphrase"
                    className="w-full px-4 py-3 rounded-xl text-sm"
                    style={{
                      background: "hsl(var(--harbor-surface-1))",
                      border: "1px solid hsl(var(--harbor-border-subtle))",
                      color: "hsl(var(--harbor-text-primary))",
                    }}
                  />
                  <input
                    type="password"
                    placeholder="New passphrase"
                    className="w-full px-4 py-3 rounded-xl text-sm"
                    style={{
                      background: "hsl(var(--harbor-surface-1))",
                      border: "1px solid hsl(var(--harbor-border-subtle))",
                      color: "hsl(var(--harbor-text-primary))",
                    }}
                  />
                  <input
                    type="password"
                    placeholder="Confirm new passphrase"
                    className="w-full px-4 py-3 rounded-xl text-sm"
                    style={{
                      background: "hsl(var(--harbor-surface-1))",
                      border: "1px solid hsl(var(--harbor-border-subtle))",
                      color: "hsl(var(--harbor-text-primary))",
                    }}
                  />
                </div>

                <button
                  className="mt-4 px-4 py-2 rounded-lg text-sm font-medium transition-colors duration-200"
                  style={{
                    background: "hsl(var(--harbor-surface-1))",
                    color: "hsl(var(--harbor-text-primary))",
                    border: "1px solid hsl(var(--harbor-border-subtle))",
                  }}
                >
                  Update Passphrase
                </button>
              </div>

              {/* Export keys */}
              <div
                className="rounded-2xl p-6"
                style={{
                  background: "hsl(var(--harbor-bg-elevated))",
                  border: "1px solid hsl(var(--harbor-border-subtle))",
                }}
              >
                <h4
                  className="font-medium mb-2"
                  style={{ color: "hsl(var(--harbor-text-primary))" }}
                >
                  Export Identity
                </h4>
                <p
                  className="text-sm mb-4"
                  style={{ color: "hsl(var(--harbor-text-secondary))" }}
                >
                  Export your encrypted identity to back it up or transfer to another device
                </p>
                <button
                  className="px-4 py-2 rounded-lg text-sm font-medium transition-colors duration-200"
                  style={{
                    background: "hsl(var(--harbor-surface-1))",
                    color: "hsl(var(--harbor-text-primary))",
                    border: "1px solid hsl(var(--harbor-border-subtle))",
                  }}
                >
                  Export Identity
                </button>
              </div>

              {/* Danger zone */}
              <div
                className="rounded-2xl p-6"
                style={{
                  background: "hsl(var(--harbor-error) / 0.05)",
                  border: "1px solid hsl(var(--harbor-error) / 0.2)",
                }}
              >
                <h4
                  className="font-medium mb-2"
                  style={{ color: "hsl(var(--harbor-error))" }}
                >
                  Danger Zone
                </h4>
                <p
                  className="text-sm mb-4"
                  style={{ color: "hsl(var(--harbor-text-secondary))" }}
                >
                  Permanently delete your identity and all associated data. This cannot be undone.
                </p>
                <button
                  className="px-4 py-2 rounded-lg text-sm font-medium transition-colors duration-200"
                  style={{
                    background: "hsl(var(--harbor-error) / 0.15)",
                    color: "hsl(var(--harbor-error))",
                    border: "1px solid hsl(var(--harbor-error) / 0.3)",
                  }}
                >
                  Delete Identity
                </button>
              </div>
            </div>
          )}

          {activeSection === "network" && (
            <div className="space-y-6">
              <div>
                <h3
                  className="text-lg font-semibold mb-1"
                  style={{ color: "hsl(var(--harbor-text-primary))" }}
                >
                  Network
                </h3>
                <p
                  className="text-sm"
                  style={{ color: "hsl(var(--harbor-text-secondary))" }}
                >
                  Configure P2P networking options
                </p>
              </div>

              <div
                className="rounded-2xl p-6"
                style={{
                  background: "hsl(var(--harbor-bg-elevated))",
                  border: "1px solid hsl(var(--harbor-border-subtle))",
                }}
              >
                <div className="flex items-center justify-between">
                  <div>
                    <h4
                      className="font-medium"
                      style={{ color: "hsl(var(--harbor-text-primary))" }}
                    >
                      Auto-start Network
                    </h4>
                    <p
                      className="text-sm mt-0.5"
                      style={{ color: "hsl(var(--harbor-text-secondary))" }}
                    >
                      Automatically connect when app starts
                    </p>
                  </div>
                  <button
                    className="w-12 h-6 rounded-full relative transition-colors duration-200"
                    style={{ background: "hsl(var(--harbor-primary))" }}
                  >
                    <div
                      className="w-5 h-5 rounded-full absolute top-0.5 right-0.5 transition-transform duration-200"
                      style={{ background: "white" }}
                    />
                  </button>
                </div>
              </div>

              <div
                className="rounded-2xl p-6"
                style={{
                  background: "hsl(var(--harbor-bg-elevated))",
                  border: "1px solid hsl(var(--harbor-border-subtle))",
                }}
              >
                <div className="flex items-center justify-between">
                  <div>
                    <h4
                      className="font-medium"
                      style={{ color: "hsl(var(--harbor-text-primary))" }}
                    >
                      mDNS Discovery
                    </h4>
                    <p
                      className="text-sm mt-0.5"
                      style={{ color: "hsl(var(--harbor-text-secondary))" }}
                    >
                      Discover peers on local network
                    </p>
                  </div>
                  <button
                    className="w-12 h-6 rounded-full relative transition-colors duration-200"
                    style={{ background: "hsl(var(--harbor-primary))" }}
                  >
                    <div
                      className="w-5 h-5 rounded-full absolute top-0.5 right-0.5 transition-transform duration-200"
                      style={{ background: "white" }}
                    />
                  </button>
                </div>
              </div>
            </div>
          )}

          {activeSection === "privacy" && (
            <div className="space-y-6">
              <div>
                <h3
                  className="text-lg font-semibold mb-1"
                  style={{ color: "hsl(var(--harbor-text-primary))" }}
                >
                  Privacy
                </h3>
                <p
                  className="text-sm"
                  style={{ color: "hsl(var(--harbor-text-secondary))" }}
                >
                  Control who can see your content
                </p>
              </div>

              <div
                className="rounded-2xl p-6"
                style={{
                  background: "hsl(var(--harbor-bg-elevated))",
                  border: "1px solid hsl(var(--harbor-border-subtle))",
                }}
              >
                <h4
                  className="font-medium mb-2"
                  style={{ color: "hsl(var(--harbor-text-primary))" }}
                >
                  Default Post Visibility
                </h4>
                <p
                  className="text-sm mb-4"
                  style={{ color: "hsl(var(--harbor-text-secondary))" }}
                >
                  Who can see your new posts by default
                </p>
                <select
                  className="w-full px-4 py-3 rounded-xl text-sm"
                  style={{
                    background: "hsl(var(--harbor-surface-1))",
                    border: "1px solid hsl(var(--harbor-border-subtle))",
                    color: "hsl(var(--harbor-text-primary))",
                  }}
                >
                  <option value="contacts">Contacts Only</option>
                  <option value="public">Anyone with the link</option>
                </select>
              </div>

              <div
                className="rounded-2xl p-6"
                style={{
                  background: "hsl(var(--harbor-bg-elevated))",
                  border: "1px solid hsl(var(--harbor-border-subtle))",
                }}
              >
                <div className="flex items-center justify-between">
                  <div>
                    <h4
                      className="font-medium"
                      style={{ color: "hsl(var(--harbor-text-primary))" }}
                    >
                      Read Receipts
                    </h4>
                    <p
                      className="text-sm mt-0.5"
                      style={{ color: "hsl(var(--harbor-text-secondary))" }}
                    >
                      Let others know when you've read their messages
                    </p>
                  </div>
                  <button
                    className="w-12 h-6 rounded-full relative transition-colors duration-200"
                    style={{ background: "hsl(var(--harbor-primary))" }}
                  >
                    <div
                      className="w-5 h-5 rounded-full absolute top-0.5 right-0.5 transition-transform duration-200"
                      style={{ background: "white" }}
                    />
                  </button>
                </div>
              </div>

              <div
                className="rounded-2xl p-6"
                style={{
                  background: "hsl(var(--harbor-bg-elevated))",
                  border: "1px solid hsl(var(--harbor-border-subtle))",
                }}
              >
                <div className="flex items-center justify-between">
                  <div>
                    <h4
                      className="font-medium"
                      style={{ color: "hsl(var(--harbor-text-primary))" }}
                    >
                      Online Status
                    </h4>
                    <p
                      className="text-sm mt-0.5"
                      style={{ color: "hsl(var(--harbor-text-secondary))" }}
                    >
                      Show when you're online to contacts
                    </p>
                  </div>
                  <button
                    className="w-12 h-6 rounded-full relative transition-colors duration-200"
                    style={{ background: "hsl(var(--harbor-primary))" }}
                  >
                    <div
                      className="w-5 h-5 rounded-full absolute top-0.5 right-0.5 transition-transform duration-200"
                      style={{ background: "white" }}
                    />
                  </button>
                </div>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
