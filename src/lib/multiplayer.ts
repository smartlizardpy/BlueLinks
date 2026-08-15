import type { FinishPayload } from "../state/game";

export type Winner = 0 | 1 | 2;

export function multiplayerWinner(playerOne: FinishPayload, playerTwo: FinishPayload): Winner {
  if (playerOne.success !== playerTwo.success) return playerOne.success ? 1 : 2;
  if (playerOne.durationMs !== playerTwo.durationMs) return playerOne.durationMs < playerTwo.durationMs ? 1 : 2;
  if (playerOne.clicks !== playerTwo.clicks) return playerOne.clicks < playerTwo.clicks ? 1 : 2;
  return 0;
}
