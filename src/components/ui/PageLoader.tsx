interface PageLoaderProps {
  label?: string;
}

export function PageLoader({ label = "Chargement..." }: PageLoaderProps) {
  return (
    <div className="flex items-center justify-center h-full p-8">
      <div className="w-8 h-8 border-2 border-accent/30 border-t-accent rounded-full animate-spin" />
      <span className="ml-3 text-sm text-fg-dim">{label}</span>
    </div>
  );
}

export default PageLoader;
