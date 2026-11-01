import type { Notices as NoticesState } from './bridge';

type NoticesProps = {
  notices: NoticesState;
};

export const Notices = ({ notices }: NoticesProps) => (
  <div class="notices">
    {notices.map((notice) => (
      <div key={notice}>{notice}</div>
    ))}
  </div>
);
