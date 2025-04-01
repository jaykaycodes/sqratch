import React from 'react'

import * as RadioGroupPrimitive from '@radix-ui/react-radio-group'
import { CircleIcon } from 'lucide-react'

import { cn } from '#/lib/utils'

function RadioGroup({
	className,
	...props
}: React.ComponentProps<typeof RadioGroupPrimitive.Root>) {
	return (
		<RadioGroupPrimitive.Root
			className={cn('grid gap-3', className)}
			data-slot="radio-group"
			{...props}
		/>
	)
}

function RadioGroupItem({
	className,
	...props
}: React.ComponentProps<typeof RadioGroupPrimitive.Item>) {
	return (
		<RadioGroupPrimitive.Item
			className={cn(
				'aspect-square size-4 shrink-0 rounded-full border border-input text-primary shadow-xs outline-ring/50 ring-ring/10 transition-[color,box-shadow] focus-visible:outline-1 focus-visible:ring-4 disabled:cursor-not-allowed disabled:opacity-50 aria-invalid:focus-visible:ring-0 dark:outline-ring/40 dark:ring-ring/20',
				className,
			)}
			data-slot="radio-group-item"
			{...props}
		>
			<RadioGroupPrimitive.Indicator
				className="relative flex items-center justify-center"
				data-slot="radio-group-indicator"
			>
				<CircleIcon className="-translate-x-1/2 -translate-y-1/2 absolute top-1/2 left-1/2 size-2 fill-primary" />
			</RadioGroupPrimitive.Indicator>
		</RadioGroupPrimitive.Item>
	)
}

export { RadioGroup, RadioGroupItem }
