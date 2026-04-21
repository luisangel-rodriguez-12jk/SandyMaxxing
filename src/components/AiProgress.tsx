import { useAiProgress, AiProgressState } from "../hooks/useAiProgress";

/**
 * Overlay de progreso para generaciones de IA.
 * Lee el estado global del hook useAiProgress, se muestra solo cuando hay
 * una generación en curso y se oculta cuando termina.
 *
 * Se monta una sola vez en Layout (punto central) para que sea visible
 * desde cualquier página que dispare una llamada a la IA.
 */
export default function AiProgress() {
  const p = useAiProgress();
  if (!p.active) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/30 backdrop-blur-sm"
      role="dialog"
      aria-live="polite"
      aria-label="Generación con IA en curso"
    >
      <div className="bg-white rounded-2xl shadow-xl border border-mint-200 w-[min(92vw,480px)] p-6 space-y-4">
        <div>
          <div className="text-xs text-mint-600 uppercase tracking-wide">
            Generación con IA
          </div>
          <div className="text-lg font-semibold text-mint-900">
            {titleFor(p)}
          </div>
        </div>

        <div className="relative h-2 rounded-full bg-mint-100 overflow-hidden">
          <div className="ai-progress-bar" />
        </div>

        <div className="flex items-center justify-between text-xs text-mint-700">
          <span>
            {prettyStage(p.stage)}
            {p.attempt && p.totalAttempts
              ? ` · Intento ${p.attempt}/${p.totalAttempts}`
              : ""}
          </span>
          <span className="tabular-nums">{p.elapsedSec}s</span>
        </div>

        <p className="text-xs text-mint-700 italic">{hintFor(p)}</p>

        {p.reason && (
          <p className="text-xs bg-amber-50 text-amber-800 border border-amber-200 p-2 rounded">
            {p.reason}
          </p>
        )}
      </div>
    </div>
  );
}

function titleFor(p: AiProgressState): string {
  if (p.label) {
    const L = p.label.charAt(0).toUpperCase() + p.label.slice(1);
    return `Generando ${L.toLowerCase()}`;
  }
  return "Generando…";
}

function prettyStage(stage: AiProgressState["stage"]): string {
  switch (stage) {
    case "start":
      return "Preparando";
    case "requesting":
      return "Consultando a la IA";
    case "validating":
      return "Validando respuesta";
    case "retrying":
      return "Corrigiendo respuesta";
    default:
      return "Generando";
  }
}

/**
 * Texto contextual rotativo: cambia según el stage o el tiempo transcurrido.
 * Reduce la ansiedad de la espera dando sensación de progreso.
 */
function hintFor(p: AiProgressState): string {
  if (p.stage === "retrying") {
    return "La IA no respetó una regla; se le pide que corrija sin volver a empezar.";
  }
  if (p.stage === "validating") {
    return "Revisando que las porciones cuadren exactamente y que no haya ingredientes prohibidos.";
  }
  const s = p.elapsedSec;
  if (s < 5) return "Preparando contexto de tu familia…";
  if (s < 15) return "La IA está proponiendo combinaciones de alimentos permitidos…";
  if (s < 30) return "Calculando las porciones por persona y redactando la preparación…";
  if (s < 45) return "Ajustando tiempos de cocción y detalles de los platillos…";
  if (s < 70) return "Casi listo, validando el resultado final…";
  return "La primera vez tarda un poco más. Los siguientes intentos son más rápidos.";
}
