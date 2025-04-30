import { use$ } from '@legendapp/state/react'

import { Tabs, TabsContent, TabsList, TabsTrigger } from '#/components/ui/tabs'
import { useProjectStore$ } from '#/providers/project'

import { WORKBENCH_TABS, type WorkbenchTab } from './misc'

export * from './misc'

export default function ProjectWorkbench() {
	const state$ = useProjectStore$().ui
	const activeTab = use$(state$.workbenchTab)

	return (
		<Tabs
			onValueChange={(value) => state$.workbenchTab.set(value as WorkbenchTab)}
			value={activeTab}
		>
			<TabsList className="rounded-none">
				{WORKBENCH_TABS.map((tab) => (
					<TabsTrigger key={tab.label} value={tab.label}>
						{tab.label}
					</TabsTrigger>
				))}
			</TabsList>
			{WORKBENCH_TABS.map((tab) => (
				<TabsContent hidden={tab.label !== activeTab} key={tab.label} value={tab.label}>
					{tab.label} content
				</TabsContent>
			))}
		</Tabs>
	)
}
