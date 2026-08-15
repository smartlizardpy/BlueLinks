import type { ButtonHTMLAttributes, PropsWithChildren } from "react";

type Props = PropsWithChildren<ButtonHTMLAttributes<HTMLButtonElement> & { variant?: "primary" | "secondary"; reveal?: boolean }>;

export function GameButton({ variant = "primary", reveal = false, className = "", ...props }: Props) {
  return <button className={`game-button ${variant} ${reveal ? "button-reveal" : ""} ${className}`} {...props} />;
}
