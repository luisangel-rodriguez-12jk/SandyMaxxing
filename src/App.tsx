import { Navigate, Route, Routes } from "react-router-dom";
import Layout from "./components/Layout";
import Usuarios from "./pages/Usuarios";
import MiPlan from "./pages/MiPlan";
import Cocinar from "./pages/Cocinar";
import Progreso from "./pages/Progreso";
import AjustesTabs from "./pages/AjustesTabs";

export default function App() {
  return (
    <Routes>
      <Route path="/" element={<Layout />}>
        <Route index element={<Navigate to="/usuarios" replace />} />
        <Route path="usuarios" element={<Usuarios />} />
        <Route path="mi-plan" element={<MiPlan />} />
        <Route path="cocinar" element={<Cocinar />} />
        <Route path="progreso" element={<Progreso />} />
        <Route path="ajustes" element={<AjustesTabs />} />
      </Route>
    </Routes>
  );
}
