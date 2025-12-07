export type CommentDto = {
  id: number;
  text: string;
  user: string;
  url_id: number;
  date: string;
};

export type CommentsPage = {
  total: number;
  items: CommentDto[];
};
