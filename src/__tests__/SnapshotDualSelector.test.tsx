import { describe, expect, it, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { SnapshotDualSelector } from "../components/ui/SnapshotDualSelector";

const groups = [
  { timestamp: "20260507_120000", backupFiles: [], keyCount: 7 },
  { timestamp: "20260506_100000", backupFiles: [], keyCount: 7 },
  { timestamp: "20260505_080000", backupFiles: [], keyCount: 7 },
];

describe("SnapshotDualSelector", () => {
  it("renders two select dropdowns labelled A and B", () => {
    render(
      <SnapshotDualSelector
        groups={groups}
        valueA=""
        valueB=""
        onChangeA={() => {}}
        onChangeB={() => {}}
      />,
    );
    expect(screen.getByLabelText(/Snapshot A/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/Snapshot B/i)).toBeInTheDocument();
  });

  it("lists all snapshots in both selects", () => {
    render(
      <SnapshotDualSelector
        groups={groups}
        valueA=""
        valueB=""
        onChangeA={() => {}}
        onChangeB={() => {}}
      />,
    );
    const optionsA = screen.getByLabelText(/Snapshot A/i).querySelectorAll("option");
    expect(optionsA.length).toBe(groups.length + 1);
  });

  it("calls onChangeA when first select changes", () => {
    const onChangeA = vi.fn();
    render(
      <SnapshotDualSelector
        groups={groups}
        valueA=""
        valueB=""
        onChangeA={onChangeA}
        onChangeB={() => {}}
      />,
    );
    fireEvent.change(screen.getByLabelText(/Snapshot A/i), {
      target: { value: "20260506_100000" },
    });
    expect(onChangeA).toHaveBeenCalledWith("20260506_100000");
  });

  it("calls onChangeB when second select changes", () => {
    const onChangeB = vi.fn();
    render(
      <SnapshotDualSelector
        groups={groups}
        valueA=""
        valueB=""
        onChangeA={() => {}}
        onChangeB={onChangeB}
      />,
    );
    fireEvent.change(screen.getByLabelText(/Snapshot B/i), {
      target: { value: "20260507_120000" },
    });
    expect(onChangeB).toHaveBeenCalledWith("20260507_120000");
  });
});
