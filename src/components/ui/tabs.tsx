import React from 'react'

import * as TabsPrimitive from '@radix-ui/react-tabs'

import { cn } from '#/lib/utils'

function Tabs({ className, ...props }: React.ComponentProps<typeof TabsPrimitive.Root>) {
	return (
		<TabsPrimitive.Root
			className={cn('flex flex-col gap-2', className)}
			data-slot="tabs"
			{...props}
		/>
	)
}

function TabsList({ className, ...props }: React.ComponentProps<typeof TabsPrimitive.List>) {
	return (
		<TabsPrimitive.List
			className={cn(
				'inline-flex h-9 w-fit items-center justify-center rounded-lg bg-muted p-1 text-muted-foreground',
				className,
			)}
			data-slot="tabs-list"
			{...props}
		/>
	)
}

function TabsTrigger({ className, ...props }: React.ComponentProps<typeof TabsPrimitive.Trigger>) {
	return (
		<TabsPrimitive.Trigger
			className={cn(
				"inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md px-2 py-1 font-medium text-sm outline-ring/50 ring-ring/10 transition-[color,box-shadow] focus-visible:outline-1 focus-visible:ring-4 disabled:pointer-events-none disabled:opacity-50 aria-invalid:focus-visible:ring-0 data-[state=active]:bg-background data-[state=active]:text-foreground data-[state=active]:shadow-sm dark:outline-ring/40 dark:ring-ring/20 [&_svg:not([class*='size-'])]:size-4 [&_svg]:pointer-events-none [&_svg]:shrink-0",
				className,
			)}
			data-slot="tabs-trigger"
			{...props}
		/>
	)
}

function TabsContent({ className, ...props }: React.ComponentProps<typeof TabsPrimitive.Content>) {
	return (
		<TabsPrimitive.Content
			className={cn(
				'flex-1 outline-ring/50 ring-ring/10 transition-[color,box-shadow] focus-visible:outline-1 focus-visible:ring-4 aria-invalid:focus-visible:ring-0 dark:outline-ring/40 dark:ring-ring/20',
				className,
			)}
			data-slot="tabs-content"
			{...props}
		/>
	)
}

export { Tabs, TabsList, TabsTrigger, TabsContent }
