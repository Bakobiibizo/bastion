/** A feed item (post with author context) */
export interface FeedItem {
  postId: string;
  authorPeerId: string;
  authorDisplayName: string | null;
  contentType: string;
  contentText: string | null;
  visibility: string;
  lamportClock: number;
  createdAt: number;
  updatedAt: number;
  isLocal: boolean;
}
