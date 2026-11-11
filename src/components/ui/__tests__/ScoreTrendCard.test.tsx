import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { ScoreTrendCard } from "../ScoreTrendCard";
import type { ScoreSnapshot } from "../../../lib/bindings";

function snap(date: string, g: number): ScoreSnapshot {
  return { date, timestamp: `${date} 12:00:00`, globalScore: g,
    scores: { dataProtection: g, services: g, hardening: g, performance: g, bloatware: g } };
}

describe("ScoreTrendCard", () => {
  it("affiche l'empty state si moins de 2 points", () => {
    render(<ScoreTrendCard history={[snap("2026-06-24", 50)]} />);
    expect(screen.getByText(/Collecte en cours/i)).toBeInTheDocument();
  });

  it("signale l'echec de lecture au lieu de dire que la collecte est en cours", () => {
    render(<ScoreTrendCard history={[]} error="Fichier illisible" />);
    expect(screen.getByRole("alert")).toHaveTextContent("Fichier illisible");
    expect(screen.queryByText(/Collecte en cours/i)).not.toBeInTheDocument();
  });

  it("propose de reessayer quand la lecture a echoue", () => {
    const onRetry = vi.fn();
    render(<ScoreTrendCard history={[]} error="Fichier illisible" onRetry={onRetry} />);
    fireEvent.click(screen.getByRole("button", { name: /réessayer/i }));
    expect(onRetry).toHaveBeenCalledTimes(1);
  });

  it("affiche les deltas 7j et 30j quand l'historique suffit", () => {
    const h = [snap("2026-05-25", 40), snap("2026-06-17", 55), snap("2026-06-24", 60)];
    render(<ScoreTrendCard history={h} />);
    expect(screen.getByText("7j")).toBeInTheDocument();
    expect(screen.getByText("30j")).toBeInTheDocument();
    expect(screen.getByText("+5")).toBeInTheDocument();
  });
});
