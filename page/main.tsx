import useSWR from "swr";
import { fetcher } from "./utils/fetcher";

type Team = {
	id: number;
	url: string;
	name: string;
	enabled: boolean;
};

async function getTeams() {
	return await fetch("/api/teams").then((res) => res.json() as Promise<Team[]>);
}

async function sendUrl(url: string) {
	await fetch("/api/teams", {
		method: "POST",
		headers: {
			"Content-Type": "application/json",
		},
		body: JSON.stringify({ url }),
	});
	return await getTeams();
}

async function flipStatus(team: Team) {
	await fetch(`/api/teams/${team.id}/flip_status`, {
		method: "PATCH",
		headers: {
			"Content-Type": "application/json",
		},
		body: JSON.stringify({ enabled: !team.enabled }),
	});
	return await getTeams();
}

export function Main() {
	const { data, error, isLoading, mutate } = useSWR<Team[]>(
		"/api/teams",
		fetcher,
	);

	async function handleSubmit(event: React.FormEvent<HTMLFormElement>) {
		event.preventDefault();
		const form = event.currentTarget;
		const formData = new FormData(form);
		const url = formData.get("url") as string;

		mutate(sendUrl(url));

		form.reset();
	}

	async function handleToggle(team: Team) {
		const newData =
			data?.map((t) => {
				if (t.id === team.id) {
					return { ...t, enabled: !t.enabled };
				}
				return t;
			}) || [];
		mutate(flipStatus(team), {
			optimisticData: newData,
			rollbackOnError: true,
		});
	}

	if (error) return <div>failed to load</div>;
	if (isLoading || !data) return <div>loading...</div>;

	return (
		<div>
			<h1>フッボー.ical</h1>
			<form onSubmit={handleSubmit}>
				<input type="text" placeholder="URLを入力" name="url" />
				<button type="submit">追加</button>
			</form>
			<table>
				<thead>
					<tr>
						<th />
						<th>チーム名</th>
					</tr>
				</thead>
				<tbody>
					{data.map((team) => (
						<tr key={team.id}>
							<td>
								<input
									type="checkbox"
									checked={team.enabled}
									onChange={() => handleToggle(team)}
									className={team.enabled ? "enabled" : "disabled"}
								/>
							</td>
							<td>{team.name}</td>
						</tr>
					))}
				</tbody>
			</table>
		</div>
	);
}
