import type { Observable } from '@legendapp/state'
import { For, use$ } from '@legendapp/state/react'
import * as Icons from 'lucide-react'

import DivButton from '#/components/div-button'
import ScrollArea from '#/components/ui/scroll-area'
import { cn } from '#/lib/utils'
import EditorStore$, { type EditorTab } from '#/stores/editor-store'

export default function EditorTabBar() {
	return (
		<div className="flex items-center gap-1 w-full">
			<div className="grow-1 h-full overflow-hidden">
				<ScrollArea scrollBarSize="sm" vertical={false}>
					<div className="tabs tabs-lift flex-nowrap" role="tablist">
						<For each={EditorStore$.tabs}>{(tab) => <TabButton tab$={tab} />}</For>
					</div>
				</ScrollArea>
			</div>

			{/* New Tab Button */}
			<button
				className="btn btn-ghost btn-xs btn-square mx-1 text-base-content/60 hover:text-base-content"
				title="New Tab"
				type="button"
			>
				<Icons.Plus className="size-4" />
			</button>
		</div>
	)
}

function TabButton({ tab$ }: { tab$: Observable<EditorTab> }) {
	const tab = use$(tab$)
	const isActive = use$(() => EditorStore$.activeTabId.get() === tab.id)

	return (
		<DivButton
			className={cn(
				'tab gap-2 px-4 flex-shrink-0',
				'!border-[0.5px] !border-b-0 !border-border',
				'[&:is(:first-child)]:!border-l-1 [&:is(:last-child)]:!border-r-1',
				isActive && 'tab-active !border-t-primary',
			)}
			onClick={() => EditorStore$.activeTabId.set(tab.id)}
			role="tab"
		>
			{/* File Type Indicator */}
			<div
				className={cn(
					'w-2 h-2 rounded-full flex-shrink-0',
					tab.resourceType === 'table' && 'bg-blue-500',
					tab.resourceType === 'file' && 'bg-green-500',
				)}
			/>

			{/* File Name */}
			<span
				className={cn(
					'text-sm truncate',
					isActive ? 'text-base-content' : 'text-base-content/70',
					tab.isDirty && 'font-medium italic',
				)}
			>
				{tab.id}
			</span>

			{/* Close Button */}
			<button
				className={cn(
					'btn btn-ghost btn-square btn-xs opacity-0 group-hover:opacity-100',
					'hover:bg-base-300 text-base-content/60 hover:text-base-content',
					isActive && 'opacity-60',
				)}
				onClick={() => EditorStore$.closeTab(tab.id)}
				title="Close"
				type="button"
			>
				{tab.isDirty ? (
					<Icons.Circle className="size-2" fill="currentColor" />
				) : (
					<Icons.X className="size-2" />
				)}
			</button>
		</DivButton>
	)
}
