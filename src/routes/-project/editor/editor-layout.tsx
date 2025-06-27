import React from 'react'

import { Plus, X } from 'lucide-react'

import { DefaultPending } from '#/components/default-screens'
import DivButton from '#/components/div-button'
import ScrollArea from '#/components/ui/scroll-area'
import { cn } from '#/lib/utils'

interface EditorTab {
	id: string
	title: string
	type: 'sql' | 'markdown' | 'json' | 'text'
	isDirty: boolean
}

// Mock data for now - will be replaced with actual state management
const MOCK_TABS = [
	{ id: '1', title: 'query-1.sql', type: 'sql', isDirty: true },
	{ id: '2', title: 'schema-analysis.md', type: 'markdown', isDirty: false },
	{ id: '3', title: 'users-table.sql', type: 'sql', isDirty: false },
	{ id: '4', title: 'users-table.sql', type: 'sql', isDirty: false },
	{ id: '5', title: 'users-table.sql', type: 'sql', isDirty: true },
	{ id: '6', title: 'users-table.sql', type: 'sql', isDirty: false },
	{ id: '7', title: 'users-table.sql', type: 'sql', isDirty: false },
	{ id: '8', title: 'users-table.sql', type: 'sql', isDirty: false },
	{ id: '9', title: 'users-table.sql', type: 'sql', isDirty: false },
	{ id: '10', title: 'users-table.sql', type: 'sql', isDirty: false },
] satisfies EditorTab[]

export default function EditorLayout() {
	const [tabs, setTabs] = React.useState<EditorTab[]>(MOCK_TABS)
	const [activeTabId, setActiveTabId] = React.useState<string>('1')

	const handleCloseTab = (tabId: string, e: React.MouseEvent) => {
		e.stopPropagation()
		const newTabs = tabs.filter((tab) => tab.id !== tabId)
		setTabs(newTabs)

		// If we closed the active tab, switch to another one
		if (activeTabId === tabId && newTabs.length > 0) {
			setActiveTabId(newTabs[0].id)
		}
	}

	const handleNewTab = () => {
		const newTab: EditorTab = {
			id: Date.now().toString(),
			title: 'untitled.sql',
			type: 'sql',
			isDirty: false,
		}
		setTabs([...tabs, newTab])
		setActiveTabId(newTab.id)
	}

	return (
		<div className="flex flex-col h-full">
			{/* Tab Bar */}
			<div className="flex items-center gap-1 w-full">
				<div className="grow-1 h-full overflow-hidden">
					<ScrollArea size="sm" vertical={false}>
						<div className="tabs tabs-lift tabs-sm flex-nowrap" role="tablist">
							{tabs.map((tab) => (
								<TabButton
									isActive={activeTabId === tab.id}
									key={tab.id}
									onClick={() => setActiveTabId(tab.id)}
									onClose={(e) => handleCloseTab(tab.id, e)}
									tab={tab}
								/>
							))}
						</div>
					</ScrollArea>
				</div>

				{/* New Tab Button */}
				<button
					className="btn btn-ghost btn-xs btn-square mx-1 text-base-content/60 hover:text-base-content"
					onClick={handleNewTab}
					title="New Tab"
				>
					<Plus className="size-4" />
				</button>
			</div>

			{/* Editor Content */}
			<div className="flex-1 overflow-hidden">
				<React.Suspense fallback={<DefaultPending showText={false} />}>
					{tabs.length > 0 ? (
						<EditorContent tab={tabs.find((tab) => tab.id === activeTabId)} />
					) : (
						<EmptyEditor onNewTab={handleNewTab} />
					)}
				</React.Suspense>
			</div>
		</div>
	)
}

interface TabButtonProps {
	tab: EditorTab
	isActive: boolean
	onClick: () => void
	onClose: (e: React.MouseEvent) => void
}

function TabButton({ tab, isActive, onClick, onClose }: TabButtonProps) {
	return (
		<DivButton
			className={cn('tab gap-2 px-4 flex-shrink-0', isActive && 'tab-active')}
			onClick={onClick}
			role="tab"
		>
			{/* File Type Indicator */}
			<div
				className={cn(
					'w-2 h-2 rounded-full flex-shrink-0',
					tab.type === 'sql' && 'bg-blue-500',
					tab.type === 'markdown' && 'bg-green-500',
					tab.type === 'json' && 'bg-yellow-500',
					tab.type === 'text' && 'bg-gray-500',
				)}
			/>

			{/* File Name */}
			<span
				className={cn(
					'text-sm truncate',
					isActive ? 'text-base-content' : 'text-base-content/70',
					tab.isDirty && 'font-medium',
				)}
			>
				{tab.title}
				{tab.isDirty && ' •'}
			</span>

			{/* Close Button */}
			<button
				className={cn(
					'btn btn-ghost btn-xs btn-square opacity-0 group-hover:opacity-100',
					'hover:bg-base-300 text-base-content/60 hover:text-base-content',
					isActive && 'opacity-60',
				)}
				onClick={onClose}
				title="Close"
			>
				<X className="size-3" />
			</button>
		</DivButton>
	)
}

interface EditorContentProps {
	tab?: EditorTab
}

function EditorContent({ tab }: EditorContentProps) {
	if (!tab) return null

	return (
		<div className="h-full p-4 bg-base-100">
			<div className="h-full border border-base-300 rounded-lg p-4 bg-base-50">
				<div className="text-sm text-base-content/60 mb-2">Editing: {tab.title}</div>

				{/* Placeholder editor - replace with actual editor component */}
				<div className="h-full bg-base-100 border border-base-300 rounded p-4 font-mono text-sm">
					{tab.type === 'sql' && (
						<div className="text-blue-600">
							<div>-- SQL Editor for {tab.title}</div>
							<div>SELECT * FROM users;</div>
							<div></div>
							<div>-- This will be replaced with Monaco Editor or similar</div>
						</div>
					)}

					{tab.type === 'markdown' && (
						<div className="text-green-600">
							<div># Markdown Editor for {tab.title}</div>
							<div></div>
							<div>This will be replaced with a markdown editor.</div>
						</div>
					)}

					{(tab.type === 'json' || tab.type === 'text') && (
						<div className="text-gray-600">
							<div>
								\/\/ {tab.type.toUpperCase()} Editor for {tab.title}
							</div>
							<div></div>
							<div>This will be replaced with appropriate editor.</div>
						</div>
					)}
				</div>
			</div>
		</div>
	)
}

function EmptyEditor({ onNewTab }: { onNewTab: () => void }) {
	return (
		<div className="h-full flex items-center justify-center bg-base-100">
			<div className="text-center">
				<div className="text-lg font-medium text-base-content/70 mb-4">No files open</div>
				<button className="btn btn-primary btn-sm" onClick={onNewTab}>
					<Plus className="size-4" />
					New File
				</button>
			</div>
		</div>
	)
}
