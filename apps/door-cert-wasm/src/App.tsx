import { useEffect, useState } from "react";
import { BrowserRouter, Link, Route, Routes } from "react-router-dom";
import { UploadPage } from "./pages/UploadPage";
import { CertificatePage } from "./pages/CertificatePage";

function effectiveTheme(): "light" | "dark" {
  const t = document.documentElement.dataset.theme;
  if (t === "light" || t === "dark") return t;
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

function ThemeToggle() {
  const [theme, setTheme] = useState(effectiveTheme);
  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    localStorage.setItem("door-cert-theme", theme);
  }, [theme]);
  return (
    <button
      className="theme-toggle"
      onClick={() => setTheme(theme === "dark" ? "light" : "dark")}
      aria-label={`Switch to ${theme === "dark" ? "light" : "dark"} theme`}
    >
      {theme === "dark" ? "light" : "dark"}
    </button>
  );
}

export default function App() {
  return (
    <BrowserRouter>
      <div className="shell">
        <header className="topbar">
          <Link to="/" className="topbar-brand">
            Door Certification <b>Bureau</b>
          </Link>
          <div className="topbar-side">
            <span className="topbar-note">in-browser · wasm engine · 20 tps</span>
            <ThemeToggle />
          </div>
        </header>
        <Routes>
          <Route path="/" element={<UploadPage />} />
          <Route path="/door/:id" element={<CertificatePage />} />
        </Routes>
      </div>
    </BrowserRouter>
  );
}
