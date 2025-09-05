import type { EventObject } from "/types.tsx";
import ChartItem from "./ChartItem.tsx";

type Props = { data: EventObject[] };

export default function Chart({ data }: Props) {
  const idMap = new Map<number, EventObject[]>();

  for (const event of data) {
    const arr = idMap.get(event.id) ?? [];
    arr.push(event);
    idMap.set(event.id, arr);
  }

  const listItems = Array.from(idMap, ([id, group]) => (
    <ChartItem key={id} data={group} />
  ));

  return (
    <div className="chart">
      Chart
      {listItems}
    </div>
  );
}
