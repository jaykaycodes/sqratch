import * as ScrollAreaPrimitive from '@radix-ui/react-scroll-area'

import { cn } from '#/lib/utils'

const barSize = {
	sm: '3px',
	md: '6px',
	lg: '9px',
} as const
type BarSize = keyof typeof barSize

interface ScrollAreaProps extends React.ComponentProps<typeof ScrollAreaPrimitive.Root> {
	horizontal?: boolean
	vertical?: boolean
	scrollBarSize?: BarSize
}

export default function ScrollArea({
	className,
	children,
	horizontal = true,
	vertical = true,
	scrollBarSize = 'md',
	...props
}: ScrollAreaProps) {
	return (
		<ScrollAreaPrimitive.Root
			className={cn('relative', className)}
			data-slot="scroll-area"
			scrollHideDelay={0}
			{...props}
		>
			<ScrollAreaPrimitive.Viewport
				className="size-full rounded-[inherit] outline-none overscroll-none"
				data-slot="scroll-area-viewport"
			>
				{children}
			</ScrollAreaPrimitive.Viewport>

			{horizontal && <ScrollBar forceMount orientation="horizontal" size={scrollBarSize} />}
			{vertical && <ScrollBar forceMount orientation="vertical" size={scrollBarSize} />}

			<ScrollAreaPrimitive.Corner />
		</ScrollAreaPrimitive.Root>
	)
}

interface ScrollBarProps
	extends React.ComponentProps<typeof ScrollAreaPrimitive.ScrollAreaScrollbar> {
	size?: BarSize
	offset?: number
}

function ScrollBar({ className, orientation = 'vertical', size = 'md', ...props }: ScrollBarProps) {
	return (
		<ScrollAreaPrimitive.ScrollAreaScrollbar
			className={cn(
				'flex touch-none p-0 select-none',
				'transition-opacity ease-out',
				'data-[state=visible]:duration-200 data-[state=visible]:opacity-100',
				'data-[state=hidden]:duration-1000 data-[state=hidden]:opacity-0',
				orientation === 'vertical' && {
					'h-full border-l border-l-transparent': true,
					'w-[3px]': size === 'sm',
					'w-[7px]': size === 'md',
					'w-[11px]': size === 'lg',
				},
				orientation === 'horizontal' && {
					'flex-col border-t border-t-transparent': true,
					'h-[3px]': size === 'sm',
					'h-[7px]': size === 'md',
					'h-[11px]': size === 'lg',
				},
				className,
			)}
			data-slot="scroll-area-scrollbar"
			orientation={orientation}
			{...props}
		>
			<ScrollAreaPrimitive.ScrollAreaThumb
				className={cn(
					'bg-base-content/25 relative flex-1 rounded-none',
					'transition-opacity duration-inherit ease-inherit',
				)}
				data-slot="scroll-area-thumb"
				forceMount
			/>
		</ScrollAreaPrimitive.ScrollAreaScrollbar>
	)
}
