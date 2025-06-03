import {
	arrow,
	FloatingArrow,
	FloatingPortal,
	flip,
	offset,
	type Placement,
	shift,
	useFloating,
	useHover,
	useInteractions,
} from "@floating-ui/react";
import React from "react";
import { cn } from "#/lib/utils";
import type { ColorVariant } from "./variants";

export interface TooltipProps {
	tip?: React.ReactNode;
	/** @default "neutral" */
	variant?: ColorVariant;
	placement?: Placement;
	/** @default 200 */
	delayMs?: number;
	/** @default 0 */
	offsetBy?: number;
	children: React.ReactElement;
}

export default function Tooltip({
	variant = "neutral",
	tip,
	placement,
	delayMs = 200,
	offsetBy = 0,
	children,
}: TooltipProps) {
	const [open, onOpenChange] = React.useState(false);

	const arrowRef = React.useRef(null);
	const { refs, floatingStyles, context } = useFloating({
		placement,
		open,
		onOpenChange,
		middleware: [
			offset(offsetBy),
			shift({ padding: 8 }),
			flip(),
			arrow({ element: arrowRef.current }),
		],
	});

	const hover = useHover(context, { enabled: !!tip, delay: delayMs });
	const interactions = useInteractions([hover]);

	return (
		<>
			{open && (
				<FloatingPortal>
					<div
						ref={refs.setFloating}
						style={floatingStyles}
						{...interactions.getFloatingProps()}
						className={cn(
							"badge z-40 bg-base-content/20 h-fit max-w-2xl text-wrap",
							variants[variant],
						)}
					>
						{tip}
						<FloatingArrow
							ref={arrowRef}
							context={context}
							className="fill-(--badge-color)"
						/>
					</div>
				</FloatingPortal>
			)}

			<div ref={refs.setReference} {...interactions.getReferenceProps()}>
				{children}
			</div>
		</>
	);
}

const variants: Record<ColorVariant, string> = {
	error: /** @tw */ "badge-error",
	accent: /** @tw */ "badge-accent",
	primary: /** @tw */ "badge-primary",
	secondary: /** @tw */ "badge-secondary",
	warning: /** @tw */ "badge-warning",
	success: /** @tw */ "badge-success",
	neutral: /** @tw */ "badge-neutral",
	info: /** @tw */ "badge-info",
};
