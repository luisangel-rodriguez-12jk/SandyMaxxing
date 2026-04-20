import Tabs from "../components/Tabs";
import { t } from "../i18n/es";
import GeneradorPlan from "./GeneradorPlan";
import DisenarComida from "./DisenarComida";
import ListaCompras from "./ListaCompras";

export default function Cocinar() {
  return (
    <div className="space-y-4">
      <header>
        <h1 className="text-2xl font-semibold">{t.nav.cocinar}</h1>
        <p className="text-sm text-mint-700">
          Plan semanal con IA, diseño de una comida para toda la familia y lista de compras automática.
        </p>
      </header>
      <Tabs
        tabs={[
          { key: "plan", label: t.tabs.planIA, render: () => <GeneradorPlan /> },
          { key: "comida", label: t.tabs.comida, render: () => <DisenarComida /> },
          { key: "compras", label: t.tabs.compras, render: () => <ListaCompras /> },
        ]}
      />
    </div>
  );
}
