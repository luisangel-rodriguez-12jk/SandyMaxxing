import { ReactNode, useState } from "react";

export interface Tab {
  key: string;
  label: string;
  render: () => ReactNode;
}

export default function Tabs({ tabs, initial }: { tabs: Tab[]; initial?: string }) {
  const [active, setActive] = useState(initial ?? tabs[0]?.key);
  const current = tabs.find((t) => t.key === active) ?? tabs[0];
  return (
    <div className="space-y-6">
      <div className="flex gap-1 border-b border-mint-200">
        {tabs.map((t) => (
          <button
            key={t.key}
            onClick={() => setActive(t.key)}
            className={
              "px-4 py-2 text-sm font-medium -mb-px border-b-2 transition " +
              (t.key === active
                ? "border-mint-600 text-mint-800"
                : "border-transparent text-mint-600 hover:text-mint-800")
            }
          >
            {t.label}
          </button>
        ))}
      </div>
      <div>{current?.render()}</div>
    </div>
  );
}
