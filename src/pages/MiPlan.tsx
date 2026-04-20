import Tabs from "../components/Tabs";
import { t } from "../i18n/es";
import Prohibidos from "./Prohibidos";
import PlanSemanal from "./PlanSemanal";
import Licuados from "./Licuados";

export default function MiPlan() {
  return (
    <div className="space-y-4">
      <header>
        <h1 className="text-2xl font-semibold">{t.nav.miPlan}</h1>
        <p className="text-sm text-mint-700">
          Configura lo que debe evitar el usuario activo, sus porciones semanales y sus licuados.
        </p>
      </header>
      <Tabs
        tabs={[
          { key: "porciones", label: t.tabs.porciones, render: () => <PlanSemanal /> },
          { key: "prohibidos", label: t.tabs.prohibidos, render: () => <Prohibidos /> },
          { key: "licuados", label: t.tabs.licuados, render: () => <Licuados /> },
        ]}
      />
    </div>
  );
}
