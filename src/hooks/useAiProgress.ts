import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";

/**
 * Etapas por las que pasa una generación con IA.
 * El backend emite eventos "ai_progress" con estos stages.
 */
export type AiProgressStage =
  | "idle"
  | "start"
  | "requesting"
  | "validating"
  | "retrying"
  | "done"
  | "error";

export type AiProgressState = {
  /** true mientras hay una generación en curso (entre 'start' y 'done'/'error'). */
  active: boolean;
  /** Etiqueta humana de qué se está generando: 'plan semanal', 'comida individual', etc. */
  label: string | null;
  /** Stage actual. */
  stage: AiProgressStage;
  /** Intento actual (1-based). */
  attempt: number | null;
  /** Total de intentos posibles. */
  totalAttempts: number | null;
  /** Motivo del reintento (si aplicable). */
  reason: string | null;
  /** Mensaje de error si stage === 'error'. */
  errorMessage: string | null;
  /** Segundos transcurridos desde 'start'. Se actualiza aprox. cada 250ms. */
  elapsedSec: number;
};

type AiProgressEventPayload = {
  stage: string;
  label?: string;
  attempt?: number;
  total?: number;
  reason?: string;
  message?: string;
};

const INITIAL: AiProgressState = {
  active: false,
  label: null,
  stage: "idle",
  attempt: null,
  totalAttempts: null,
  reason: null,
  errorMessage: null,
  elapsedSec: 0,
};

/**
 * Hook que escucha los eventos 'ai_progress' emitidos por el backend
 * y expone un estado reactivo para mostrar una barra de progreso.
 *
 * Uso típico: montar una sola instancia en Layout para que el overlay
 * sea global.
 */
export function useAiProgress(): AiProgressState {
  const [state, setState] = useState<AiProgressState>(INITIAL);
  const startedAtRef = useRef<number | null>(null);

  // Escuchamos los eventos del backend.
  useEffect(() => {
    let disposed = false;
    const unlistenPromise = listen<AiProgressEventPayload>("ai_progress", (ev) => {
      if (disposed) return;
      const p = ev.payload;
      setState((prev) => {
        const next: AiProgressState = { ...prev };
        switch (p.stage) {
          case "start":
            startedAtRef.current = Date.now();
            next.active = true;
            next.label = p.label ?? null;
            next.stage = "start";
            next.attempt = null;
            next.totalAttempts = null;
            next.reason = null;
            next.errorMessage = null;
            next.elapsedSec = 0;
            break;
          case "requesting":
            next.stage = "requesting";
            next.attempt = p.attempt ?? prev.attempt;
            next.totalAttempts = p.total ?? prev.totalAttempts;
            break;
          case "validating":
            next.stage = "validating";
            next.attempt = p.attempt ?? prev.attempt;
            break;
          case "retrying":
            next.stage = "retrying";
            next.attempt = p.attempt ?? prev.attempt;
            next.reason = p.reason ?? null;
            break;
          case "done":
            startedAtRef.current = null;
            next.active = false;
            next.stage = "done";
            break;
          case "error":
            startedAtRef.current = null;
            next.active = false;
            next.stage = "error";
            next.errorMessage = p.message ?? "Error desconocido";
            break;
          default:
            // stage desconocido — lo ignoramos
            break;
        }
        return next;
      });
    });

    return () => {
      disposed = true;
      unlistenPromise.then((fn) => fn()).catch(() => {});
    };
  }, []);

  // Tick del contador de segundos mientras hay una generación activa.
  useEffect(() => {
    if (!state.active) return;
    const id = setInterval(() => {
      const startedAt = startedAtRef.current;
      if (startedAt == null) return;
      const elapsed = Math.floor((Date.now() - startedAt) / 1000);
      setState((prev) =>
        prev.elapsedSec === elapsed ? prev : { ...prev, elapsedSec: elapsed },
      );
    }, 250);
    return () => clearInterval(id);
  }, [state.active]);

  return state;
}
