import type * as React from 'react'

import { Slot } from '@radix-ui/react-slot'
import { cva, type VariantProps } from 'class-variance-authority'

import { cn } from '#/lib/utils'

const buttonVariants = cva(
	"inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md font-medium text-sm outline-ring/50 ring-ring/10 transition-[color,box-shadow] focus-visible:outline-1 focus-visible:ring-4 disabled:pointer-events-none disabled:opacity-50 aria-invalid:focus-visible:ring-0 dark:outline-ring/40 dark:ring-ring/20 [&_svg:not([class*='size-'])]:size-4 [&_svg]:pointer-events-none [&_svg]:shrink-0",
	{
		variants: {
			variant: {
				default: 'bg-primary text-primary-foreground shadow-sm hover:bg-primary/90',
				destructive: 'bg-destructive text-destructive-foreground shadow-xs hover:bg-destructive/90',
				outline:
					'border border-input bg-background shadow-xs hover:bg-accent hover:text-accent-foreground',
				secondary: 'bg-secondary text-secondary-foreground shadow-xs hover:bg-secondary/80',
				ghost: 'hover:bg-accent hover:text-accent-foreground',
				link: 'text-primary underline-offset-4 hover:underline',
			},
			size: {
				default: 'h-9 px-4 py-2 has-[>svg]:px-3',
				sm: 'h-8 rounded-md px-3 has-[>svg]:px-2.5',
				lg: 'h-10 rounded-md px-6 has-[>svg]:px-4',
				icon: 'size-9',
			},
		},
		defaultVariants: {
			variant: 'default',
			size: 'default',
		},
	},
)

type ButtonVariantProps = VariantProps<typeof buttonVariants>
type ButtonButtonProps = ButtonVariantProps & React.ComponentProps<'button'>
type ButtonDivProps = ButtonVariantProps & React.ComponentProps<'div'> & { asDiv: true }
type ButtonChildProps = ButtonVariantProps & React.ComponentProps<typeof Slot> & { asChild: true }

function Button({
	variant,
	size,
	...rest
}: ButtonButtonProps | ButtonDivProps | ButtonChildProps) {
	const className = cn(buttonVariants({ variant, size, className: rest.className }))

	if ('asDiv' in rest) {
		const { asDiv, ...props } = rest
		return (
			<div
				onKeyUp={(e) => {
					if (e.key === 'Enter' || e.key === ' ') {
						e.preventDefault()
						// biome-ignore lint/suspicious/noExplicitAny: <explanation>
						rest.onClick?.(e as any)
					}
				}}
				role="button"
				tabIndex={0}
				{...props}
				className={className}
			/>
		)
	}


	if ('asChild' in rest) {
		const { asChild, ...props } = rest
		return <Slot data-slot="button" {...props} className={className} />
	}

	return <button {...rest} className={className} />
}

export { Button, type ButtonVariantProps, buttonVariants }
