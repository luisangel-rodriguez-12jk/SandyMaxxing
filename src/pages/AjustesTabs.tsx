import Tabs from "../components/Tabs";
import { t } from "../i18n/es";
import Ajustes from "./Ajustes";
import Alimentos from "./Alimentos";

export default function AjustesTabs() {
  return (
    <div className="space-y-4">
      <header>
        <h1 className="text-2xl font-semibold">{t.nav.ajustes}</h1>
        <p className="text-sm text-mint-700">
          Clave de OpenAI y tabla de equivalencias de alimentos.
        </p>
      </header>
      <Tabs
        tabs={[
          { key: "clave", label: t.tabs.claveIA, render: () => <Ajustes /> },
          { key: "equiv", label: t.tabs.equivalencias, render: () => <Alimentos /> },
        ]}
      />
    </div>
  );
}
