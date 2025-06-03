import { type Observable } from "@legendapp/state";
import { For, Show } from "@legendapp/state/react";

import WorkbenchItemRow from "./item";
import type { WorkbenchItem } from "./misc";

interface Props {
	items$: Observable<WorkbenchItem[]>;
	emptyText: string;
	state: "ready" | "loading" | "pending";
}

export default function WorkbenchSection({ items$, emptyText, state }: Props) {
	if (state === "loading") return <Loading />;

	return (
		<div className="h-full overflow-x-clip overflow-y-auto">
			<Show else={<Empty text={emptyText} />} ifReady={items$}>
				<For
					each={items$}
					item={WorkbenchItemRow}
					itemProps={{ isPending: state === "pending" }}
				/>
			</Show>
		</div>
	);
}

function Empty({ text }: { text: string }) {
	return (
		<div className="flex p-2 items-center justify-center h-16 text-muted-foreground">
			{text}
		</div>
	);
}

function Loading() {
	return (
		<div className="flex flex-col gap-2 p-2">
			<div className="skeleton w-full h-6" />
			<div className="skeleton w-full h-6" />
			<div className="skeleton w-full h-6" />
			<div className="skeleton w-full h-6" />
			<div className="skeleton w-full h-6" />
			<div className="skeleton w-full h-6" />
			<div className="skeleton w-full h-6" />
		</div>
	);
}
