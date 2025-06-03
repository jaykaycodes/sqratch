import { $React } from '@legendapp/state/react-web'

import { cn } from '#/lib/utils'
import ui$ from '#/stores/ui-store'

import DatabaseTab from './database-tab'
import { WORKBENCH_TABS, type WorkbenchTab } from './misc'

export * from './misc'

export default function ProjectWorkbench() {
	return (
		<div className="flex flex-col h-full">
			<div className="tabs tabs-border w-full overflow-auto justify-center">
				{WORKBENCH_TABS.map((tab) => (
					<label className="tab" key={tab.label}>
						<$React.input
							$checked={() => ui$.workbench.activeTab.get() === tab.label}
							aria-label={tab.label}
							name="workbench-tab"
							onChange={(e) => {
								if (e.target.checked) ui$.workbench.activeTab.set(tab.label)
							}}
							type="radio"
						/>
						<tab.Icon className="size-5" />
					</label>
				))}
			</div>

			<WorkbenchTabContent tab="Database" />
			<WorkbenchTabContent tab="Queries" />
		</div>
	)
}

function WorkbenchTabContent({ tab }: { tab: WorkbenchTab }) {
	return (
		<$React.div
			$className={() =>
				cn('overflow-auto flex-1 p-2', tab !== ui$.workbench.activeTab.get() && 'hidden')
			}
		>
			{tab === 'Database' ? <DatabaseTab /> : <QueriesTab />}
		</$React.div>
	)
}

function QueriesTab() {
	return <div>Queries Tab</div>
}
