import { observer, use$ } from '@legendapp/state/react'
import type { ConditionalExcept } from 'type-fest'

import Icons from '#/components/icons'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '#/components/ui/collapsible'
import { ResizablePanel } from '#/components/ui/resizable'
import { ScrollArea } from '#/components/ui/scroll-area'
import type { ProjectStore } from '#/providers/project'

import WorkbenchGroupRow from './group'
import WorkbenchItemRow from './item'

type AllSectionStates = ConditionalExcept<ProjectStore['workbench'], Function>
type AnySectionState = AllSectionStates[keyof AllSectionStates]

interface Props {
	state$: AnySectionState
	title: string
	emptyText?: string
}

const collapsedHeightPx = 24

const WorkbenchSection = observer(function WorkbenchSection({
	state$,
	title,
	emptyText = 'No items found',
}: Props) {
	const open = use$(state$.open)
	const size = use$(state$.heightPct)
	const order = use$(state$.order)
	const items = use$(state$.items)

	const Icon = open ? Icons.ChevronDown : Icons.ChevronRight

	return (
		<ResizablePanel
			defaultSize={open ? size : 0}
			id={title}
			maxSize={80}
			minSize={open ? 10 : 0}
			order={order}
			style={
				open
					? undefined
					: {
							flexGrow: 0,
							flexBasis: `${collapsedHeightPx}px`,
						}
			}
		>
			<Collapsible
				className="overflow-hidden h-full"
				onOpenChange={(state) => state$.open.set(state)}
				open={open}
			>
				<CollapsibleTrigger asChild>
					<div
						className="flex w-full items-center gap-1 px-2 py-1"
						style={{ height: collapsedHeightPx }}
					>
						<Icon className="text-muted-foreground" size={14} />
						<span className="font-medium text-xs uppercase">{title}</span>
					</div>
				</CollapsibleTrigger>
				<CollapsibleContent className="h-full overflow-hidden">
					<ScrollArea className="h-full overflow-auto">
						{items && items.length > 0 ? (
							<div className="space-y-0.5">
								{items.map((item) =>
									item.type === 'Folder' || item.type === 'Schema' ? (
										<WorkbenchGroupRow group={item} key={item.id} />
									) : (
										<WorkbenchItemRow item={item} key={item.id} />
									),
								)}
							</div>
						) : (
							<Empty text={emptyText} />
						)}
					</ScrollArea>
				</CollapsibleContent>
			</Collapsible>
		</ResizablePanel>
	)
})

export default WorkbenchSection

function Empty({ text }: { text: string }) {
	return (
		<div className="flex items-center justify-center h-16 text-muted-foreground">
			<span className="text-xs">{text}</span>
		</div>
	)
}
