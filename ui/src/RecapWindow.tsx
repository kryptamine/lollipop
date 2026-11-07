import { useRecap } from './use-recap';
import { LessonList } from './LessonList';
import { RecapChart } from './RecapChart';
import { TrendChart } from './TrendChart';
import { RecapHeader } from './RecapHeader';
import { csPoints, goldPoints } from './recap-points';

export const RecapWindow = () => {
  const recap = useRecap();

  if (recap === undefined) {
    return <main class="recap-empty">The Recap shows here once a game ends.</main>;
  }

  const { champion, minutes, deaths, slowFarm, length, trend, lessons } = recap;
  const gold = goldPoints(minutes);

  return (
    <main class="recap">
      <RecapHeader recap={recap} />
      <RecapChart
        title="CS"
        points={csPoints(minutes)}
        deaths={deaths}
        slowFarm={slowFarm}
        length={length}
      />
      {trend.length > 1 && <TrendChart champion={champion} trend={trend} />}
      {gold.length > 0 && (
        <RecapChart
          title="Gold earned"
          points={gold}
          deaths={deaths}
          slowFarm={slowFarm}
          length={length}
        />
      )}
      <LessonList lessons={lessons} />
    </main>
  );
};
