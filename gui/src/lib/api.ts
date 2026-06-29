// Typed wrappers over the Tauri commands exposed by src-tauri/src/main.rs.
import { invoke } from "@tauri-apps/api/core";

export interface GridConfig {
  map: string;
  algorithm: string;
  heuristic: string;
  diagonal: boolean;
  weight: number;
  beamWidth?: number | null;
  depthLimit?: number | null;
  maxNodes?: number | null;
}

export interface Step {
  r: number;
  c: number;
  g: number;
  frontier: number;
}

export interface ResultDto {
  found: boolean;
  // serde serializes f64::INFINITY as null, so an unreachable cost arrives null.
  cost: number | null;
  nodesExpanded: number;
  nodesGenerated: number;
  maxFrontierSize: number;
  stopReason: string;
  path: [number, number][];
  trace: Step[];
  tree: [[number, number], [number, number]][];
}

export interface CompareRow {
  algorithm: string;
  found: boolean;
  cost: number | null;
  nodesExpanded: number;
  maxFrontierSize: number;
  pathLen: number;
}

export const runGrid = (config: GridConfig) => invoke<ResultDto>("run_grid", { config });

export const compareGrid = (config: GridConfig) =>
  invoke<CompareRow[]>("compare_grid", { config });

export const randomMaze = (width: number, height: number, density: number, seed: number) =>
  invoke<string>("random_maze_map", { width, height, density, seed });

export const sampleMaze = (name: string) => invoke<string>("sample_maze", { name });
