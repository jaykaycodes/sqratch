import React from 'react'

import { Show } from '@legendapp/state/react'

import { DefaultPending } from '#/components/default-screens'
import EditorStore$ from '#/stores/editor-store'

import EditorTabBar from './editor-tab-bar'

export default function EditorLayout() {
	return (
		<div className="flex flex-col h-full">
			{/* Tab Bar */}
			<EditorTabBar />

			{/* Editor Content */}
			<div className="flex-1 overflow-hidden">
				<React.Suspense fallback={<DefaultPending showText={false} />}>
					<Show else={<EmptyEditor />} if={() => EditorStore$.tabs.size > 0}>
						<EditorContent />
					</Show>
				</React.Suspense>
			</div>
		</div>
	)
}

function EditorContent() {
	return <div>EditorContent</div>
}

function EmptyEditor() {
	return <div>EmptyEditor</div>
}
