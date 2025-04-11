import type { Observable } from '@legendapp/state'
import { For, Show, use$ } from '@legendapp/state/react'

import Icons from '#/components/icons'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '#/components/ui/collapsible'
import { ScrollArea } from '#/components/ui/scroll-area'
import { cn } from '#/lib/utils'

import WorkbenchGroupRow, { type WorkbenchGroup } from './group'
import WorkbenchItemRow, { type WorkbenchItem } from './item'

interface WorkbenchSectionProps {
	title: string
	items$: Observable<(WorkbenchItem | WorkbenchGroup)[]>
	emptyText?: string
	defaultOpen?: boolean
	className?: string
}

export default function WorkbenchSection({
	title,
	items$,
	emptyText = 'No items found',
	defaultOpen = true,
	className,
}: WorkbenchSectionProps) {
	return (
		<Collapsible className={cn('border-b', className)} defaultOpen={defaultOpen}>
			<CollapsibleTrigger asChild>
				<div className="flex h-6 w-full items-center justify-between px-2 py-0.5 hover:bg-muted/50 cursor-pointer">
					<span className="font-medium text-xs uppercase">{title}</span>
					<Icons.ChevronDown className="text-muted-foreground" size={14} />
				</div>
			</CollapsibleTrigger>
			<CollapsibleContent>
				<ScrollArea className="px-1 py-1" style={{ height: 'var(--section-height, 180px)' }}>
					<Show else={<Empty text={emptyText} />} if={items$.length}>
						{() => (
							<div className="space-y-0.5">
								<For each={items$} item={Row} />
							</div>
						)}
					</Show>
				</ScrollArea>
			</CollapsibleContent>
		</Collapsible>
	)
}

function Row({ item$ }: { item$: Observable<WorkbenchItem | WorkbenchGroup> }) {
	const item = use$(item$)
	return item.type === 'Folder' || item.type === 'Schema' ? (
		<WorkbenchGroupRow group={item} />
	) : (
		<WorkbenchItemRow item={item as WorkbenchItem} />
	)
}

function Empty({ text }: { text: string }) {
	return (
		<div className="flex items-center justify-center h-16 text-muted-foreground">
			<span className="text-xs">{text}</span>
		</div>
	)
}
