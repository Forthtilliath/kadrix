/**
 * Les composants de forth-ui passent en sombre via la classe `.dark` : elle suit
 * ici le thème de Windows. Posée depuis un module, car la CSP refuse les scripts inline.
 */
export function followSystemTheme(): void {
  const query = window.matchMedia("(prefers-color-scheme: dark)");
  const apply = () => document.documentElement.classList.toggle("dark", query.matches);
  apply();
  query.addEventListener("change", apply);
}
