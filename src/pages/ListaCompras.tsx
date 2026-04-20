import { useMutation, useQuery } from "@tanstack/react-query";
import { useMemo, useState } from "react";
import { save } from "@tauri-apps/plugin-dialog";
import { writeFile } from "@tauri-apps/plugin-fs";
import { api, ShoppingItem } from "../api/invoke";
import { t } from "../i18n/es";

export default function ListaCompras() {
  const { data: users } = useQuery({ queryKey: ["users"], queryFn: api.usersList });
  const [selected, setSelected] = useState<number[]>([]);
  const [items, setItems] = useState<ShoppingItem[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);

  const build = useMutation({
    mutationFn: () => api.shoppingBuild(selected, null),
    onSuccess: (r) => {
      setItems(r);
      setError(null);
    },
    onError: (e: any) => setError(String(e)),
  });

  const byGroup = useMemo(() => {
    const m = new Map<string, ShoppingItem[]>();
    items.forEach((i) => {
      const a = m.get(i.group_name) ?? [];
      a.push(i);
      m.set(i.group_name, a);
    });
    return Array.from(m.entries());
  }, [items]);

  function onBuild() {
    setError(null);
    if (selected.length === 0) {
      setError("Selecciona al menos un usuario para calcular la lista de compras.");
      return;
    }
    build.mutate();
  }

  async function exportPdf() {
    setError(null);
    if (items.length === 0) {
      setError("Calcula la lista antes de exportar el PDF.");
      return;
    }
    try {
      const bytes = await api.pdfShopping(items, "Lista de compras");
      const path = await save({
        filters: [{ name: "PDF", extensions: ["pdf"] }],
        defaultPath: "lista-compras.pdf",
      });
      if (path) {
        await writeFile(path, new Uint8Array(bytes));
        setMessage("✓ PDF exportado");
        setTimeout(() => setMessage(null), 2500);
      }
    } catch (e: any) {
      setError(String(e));
    }
  }

  return (
    <div className="space-y-6 max-w-4xl">
      <p className="text-sm text-mint-700">{t.compras.subtitulo}</p>

      <div className="card space-y-3">
        <div>
          <span className="label">Usuarios</span>
          <div className="flex flex-wrap gap-2 mt-1">
            {users && users.length > 0 ? (
              users.map((u) => {
                const on = selected.includes(u.id);
                return (
                  <button
                    key={u.id}
                    onClick={() =>
                      setSelected((s) =>
                        on ? s.filter((i) => i !== u.id) : [...s, u.id],
                      )
                    }
                    className={
                      on
                        ? "rounded-full px-3 py-1 text-xs font-medium bg-mint-600 text-white"
                        : "rounded-full px-3 py-1 text-xs font-medium bg-mint-100 text-mint-800 hover:bg-mint-200"
                    }
                  >
                    {u.name}
                  </button>
                );
              })
            ) : (
              <span className="text-xs text-mint-700">
                Aún no hay usuarios. Crea uno en la sección Usuarios.
              </span>
            )}
          </div>
        </div>

        {error && <p className="text-sm text-red-600">{error}</p>}
        {message && <p className="text-sm text-green-700">{message}</p>}

        <div className="flex justify-end gap-2">
          <button
            className="btn-primary"
            disabled={build.isPending}
            onClick={onBuild}
          >
            {build.isPending ? t.common.cargando : t.compras.generar}
          </button>
          <button
            className="btn-ghost"
            disabled={items.length === 0}
            onClick={exportPdf}
          >
            {t.common.exportar}
          </button>
        </div>
      </div>

      {items.length === 0 ? (
        <p className="text-sm text-mint-700">
          Aún no se ha calculado ninguna lista. Selecciona usuarios y presiona{" "}
          <strong>{t.compras.generar}</strong>.
        </p>
      ) : (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
          {byGroup.map(([gname, list]) => (
            <div key={gname} className="card">
              <h3 className="font-semibold text-mint-700 mb-2">{gname}</h3>
              <ul className="text-sm space-y-1">
                {list.map((it, i) => (
                  <li key={i} className="flex justify-between">
                    <span>{it.name}</span>
                    <span className="text-mint-700">
                      {it.quantity.toFixed(2)} {it.unit}
                    </span>
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
