import { useQuery } from "@tanstack/react-query";
import { api } from "../api/invoke";
import { useActiveUser } from "../state/activeUser";
import { t } from "../i18n/es";
import LineChart from "../components/charts/LineChart";
import Tabs from "../components/Tabs";
import Mediciones from "./Mediciones";
import Historial from "./Historial";

function Graficas() {
  const activeUserId = useActiveUser((s) => s.activeUserId);
  const { data: list } = useQuery({
    queryKey: ["measurements", activeUserId],
    queryFn: () => api.measurementsList(activeUserId!),
    enabled: activeUserId != null,
  });

  if (!activeUserId) {
    return (
      <div className="card border-l-4 border-l-amber-400 bg-amber-50">
        <p className="text-sm text-amber-800">
          {t.common.seleccionarUsuario} para registrar y visualizar su progreso.
        </p>
      </div>
    );
  }

  const dates = list?.map((m) => m.date) ?? [];
  const hasData = (list?.length ?? 0) > 0;

  return (
    <div className="space-y-6">
      {hasData ? (
        <div className="grid grid-cols-1 xl:grid-cols-2 gap-4">
          <div className="card">
            <LineChart
              title={t.measurements.peso}
              dates={dates}
              values={list!.map((m) => m.weight)}
              color="#306d54"
              unit="kg"
            />
          </div>
          <div className="card">
            <LineChart
              title={t.measurements.cintura}
              dates={dates}
              values={list!.map((m) => m.waist_cm)}
              color="#63a485"
              unit="cm"
            />
          </div>
          <div className="card">
            <LineChart
              title={t.measurements.abdomen}
              dates={dates}
              values={list!.map((m) => m.abdomen_cm)}
              color="#8ec0a7"
              unit="cm"
            />
          </div>
          <div className="card">
            <LineChart
              title={t.measurements.cadera}
              dates={dates}
              values={list!.map((m) => m.hip_cm)}
              color="#428a6a"
              unit="cm"
            />
          </div>
        </div>
      ) : (
        <div className="card border-l-4 border-l-amber-400 bg-amber-50">
          <p className="text-sm text-amber-800">
            Aún no hay mediciones registradas. Agrega la primera abajo y verás las gráficas
            automáticamente.
          </p>
        </div>
      )}

      <Mediciones />
    </div>
  );
}

export default function Progreso() {
  return (
    <div className="space-y-4">
      <header>
        <h1 className="text-2xl font-semibold">{t.nav.progreso}</h1>
        <p className="text-sm text-mint-700">
          Registra mediciones semanales, observa la evolución y consulta el historial.
        </p>
      </header>
      <Tabs
        tabs={[
          { key: "graficas", label: t.tabs.graficas, render: () => <Graficas /> },
          { key: "historial", label: t.tabs.historial, render: () => <Historial /> },
        ]}
      />
    </div>
  );
}
