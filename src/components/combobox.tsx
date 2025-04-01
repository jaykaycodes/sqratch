import React from 'react'

import { compareItems, rankItem } from '@tanstack/match-sorter-utils'

import { Button } from '#/components/ui/button'
import {
	Command,
	CommandEmpty,
	CommandGroup,
	CommandInput,
	CommandItem,
	CommandList,
	CommandSeparator,
} from '#/components/ui/command'
import { Popover, PopoverContent, PopoverTrigger } from '#/components/ui/popover'
import { cn } from '#/lib/utils'

import Icons from './icons'
import { Skeleton } from './ui/skeleton'

export interface ComboboxOption<T extends string | number> {
	id: T
	label: string
}

export type ComboboxActions = Record<
	string,
	React.ReactNode | ((query: string) => React.ReactNode) | null
>

interface ComboboxPropsBase<
	T extends string | number,
	TActions extends ComboboxActions = Record<string, never>,
> {
	options: ComboboxOption<T>[]
	className?: string
	inputClassName?: string
	placeholder?: string
	disabled?: boolean
	actions?: TActions
	onAction?: (action: keyof TActions, query: string) => void
	isLoading?: boolean
	modal?: boolean
}

interface ComboboxPropsSingle<T extends string | number, TActions extends ComboboxActions>
	extends ComboboxPropsBase<T, TActions> {
	multi?: false
	selected?: T | null
	onChange?: (value: ComboboxOption<T> | undefined) => void
}

interface ComboboxPropsMulti<T extends string | number, TActions extends ComboboxActions>
	extends ComboboxPropsBase<T, TActions> {
	multi: true
	selected?: T[] | null
	onChange?: (value: ComboboxOption<T>[]) => void
}

export type ComboboxProps<T extends string | number, TActions extends ComboboxActions> =
	| ComboboxPropsSingle<T, TActions>
	| ComboboxPropsMulti<T, TActions>

export function Combobox<T extends string | number, TActions extends ComboboxActions>({
	options,
	placeholder = 'Select option...',
	className,
	inputClassName,
	disabled = false,
	actions,
	onAction,
	isLoading,
	modal = false,
	...props
}: ComboboxProps<T, TActions>) {
	const [open, setOpen] = React.useState(false)
	const [query, setQuery] = React.useState('')

	// Only maintain internal state if props.selected is not provided
	const [internalSelectedIds, setInternalSelectedIds] = React.useState<T[]>([])
	// Use props.selected if provided, otherwise use internal state
	const selectedIds =
		props.selected !== undefined && props.selected !== null
			? props.multi
				? props.selected
				: [props.selected]
			: internalSelectedIds

	function handleChange(value: ComboboxOption<T>) {
		if (props.multi) {
			const newIds = selectedIds.includes(value.id)
				? selectedIds.filter((id) => id !== value.id)
				: [...selectedIds, value.id]

			props.onChange?.(
				newIds.map((id) => options.find((o) => o.id === id)).filter(Boolean) as ComboboxOption<T>[],
			)
			setInternalSelectedIds(newIds)
		} else {
			props.onChange?.(value)
			setInternalSelectedIds([value.id])
			setOpen(false)
		}
	}

	const filteredOptions = React.useMemo(() => {
		if (!query) return options

		return options
			.map((item) => ({ item, rank: rankItem(item.label, query) }))
			.filter((o) => o.rank.passed)
			.sort((a, b) => compareItems(a.rank, b.rank))
			.map((o) => o.item)
	}, [options, query])

	const listContainerRef = React.useRef<HTMLDivElement>(null)
	const onQueryChange = (search: string) => {
		setQuery(search)
		listContainerRef.current?.scrollTo({ top: 0, behavior: 'instant' })
	}

	return (
		<Popover modal={modal} onOpenChange={setOpen} open={open}>
			<PopoverTrigger asChild>
				<Button
					aria-expanded={open}
					className={cn('flex min-w-[100px] justify-between', className)}
					disabled={isLoading || disabled}
					role="combobox"
					variant="outline"
				>
					{isLoading ? (
						<Skeleton className="h-4 w-full" />
					) : (
						<>
							{selectedIds.length > 0 ? (
								<div className="relative mr-auto flex flex-grow flex-wrap items-center overflow-hidden">
									<span>
										{selectedIds.map((s) => options.find((o) => o.id === s)?.label).join(', ')}
									</span>
								</div>
							) : (
								<span className="text-muted-foreground">{placeholder}</span>
							)}
							<Icons.ChevronsUpDown className="h-4 w-4 shrink-0 opacity-50" />
						</>
					)}
				</Button>
			</PopoverTrigger>
			<PopoverContent className={cn('p-0', className)}>
				<Command shouldFilter={false}>
					<CommandInput
						className={inputClassName}
						onValueChange={onQueryChange}
						placeholder={isLoading ? 'Loading...' : 'Search...'}
						value={query}
					/>
					<CommandList ref={listContainerRef}>
						<CommandEmpty>No results found.</CommandEmpty>
						{actions && Object.values(actions).filter(Boolean).length > 0 && (
							<>
								<CommandGroup>
									{Object.entries(actions).map(([action, actionFn]) => {
										if (!actionFn) return null
										return (
											<CommandItem key={action} onSelect={() => onAction?.(action, query)}>
												{typeof actionFn === 'function' ? actionFn(query) : actionFn}
											</CommandItem>
										)
									})}
								</CommandGroup>
								{filteredOptions.length > 0 && <CommandSeparator alwaysRender />}
							</>
						)}

						<CommandGroup>
							{filteredOptions.map((option) => (
								<CommandItem
									disabled={disabled}
									key={option.id}
									onSelect={() => handleChange(option)}
									value={option.id.toString()}
								>
									{props.multi && (
										<Icons.Check
											className={cn(
												'mr-2 h-4 w-4',
												selectedIds.includes(option.id) ? 'opacity-100' : 'opacity-0',
											)}
										/>
									)}
									{option.label}
								</CommandItem>
							))}
						</CommandGroup>
					</CommandList>
				</Command>
			</PopoverContent>
		</Popover>
	)
}
