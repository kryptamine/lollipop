import type { Lessons, LessonsOffReason } from './recap-types';

const OFF: Record<LessonsOffReason, string> = {
  noKey: 'No lessons: add an OpenAI key in Settings → Advice.',
  keyRefused: 'No lessons: OpenAI refused the saved key.',
  adviceWidgetOff: 'No lessons: the Advice Widget is off.',
  switchedOff: 'Lessons are off in Settings → Advice.',
  recapOff: 'No lessons: the Recap window is off.',
};

const NOTE = {
  asking: 'Asking for lessons…',
  empty: 'No lessons this time: nothing in this game stood out.',
  failed: 'Could not get lessons from OpenAI.',
} as const;

type LessonListProps = {
  lessons: Lessons;
};

const noteOf = (
  lessons: Exclude<
    Lessons,
    {
      state: 'given';
    }
  >,
): string => (lessons.state === 'off' ? OFF[lessons.detail] : NOTE[lessons.state]);

export const LessonList = ({ lessons }: LessonListProps) => (
  <>
    <div class="caption">Lessons</div>
    <div class="group">
      {lessons.state === 'given' ? (
        <ol class="lessons">
          {lessons.detail.map((lesson) => (
            <li key={lesson}>{lesson}</li>
          ))}
        </ol>
      ) : (
        <div class="row recap-note">{noteOf(lessons)}</div>
      )}
    </div>
  </>
);
