import React from 'react'

import { Slot } from '@radix-ui/react-slot'
import { cva, type VariantProps } from 'class-variance-authority'

import { cn } from '#/lib/utils'

const badgeVariants = cva(
	'inline-flex w-fit shrink-0 items-center justify-center gap-1 whitespace-nowrap rounded-md border px-2 py-0.5 font-semibold text-xs outline-ring/50 ring-ring/10 transition-[color,box-shadow] focus-visible:outline-1 focus-visible:ring-4 aria-invalid:focus-visible:ring-0 dark:outline-ring/40 dark:ring-ring/20 [&>svg]:pointer-events-none [&>svg]:size-3',
	{
		variants: {
			variant: {
				default:
					'border-transparent bg-primary text-primary-foreground shadow-sm [a&]:hover:bg-primary/90',
				secondary:
					'border-transparent bg-secondary text-secondary-foreground [a&]:hover:bg-secondary/90',
				destructive:
					'border-transparent bg-destructive text-destructive-foreground shadow-sm [a&]:hover:bg-destructive/90',
				outline: 'text-foreground [a&]:hover:bg-accent [a&]:hover:text-accent-foreground',
			},
		},
		defaultVariants: {
			variant: 'default',
		},
	},
)

function Badge({
	className,
	variant,
	asChild = false,
	...props
}: React.ComponentProps<'span'> & VariantProps<typeof badgeVariants> & { asChild?: boolean }) {
	const Comp = asChild ? Slot : 'span'

	return <Comp className={cn(badgeVariants({ variant }), className)} data-slot="badge" {...props} />
}

export { Badge, badgeVariants }
