<script lang="ts">
	import { invoke } from '@tauri-apps/api';
	import { onDestroy, onMount } from 'svelte';

	interface WeatherData {
		wind_speed: number;
		wind_direction: number;
		relative_humidity: number;
		temperature: number;
		atmospheric_pressure: number;
		dhi: number;
		ghi: number;
		dni: number;
		precipitation: number;
		gti: number;
	}

	interface TrackerData {
		angle: number;
		motor_current: number;
		target_angle: number;
		temperature: number;
		state_of_charge: number;
	}

	interface InverterData {
		number_of_string: number;
		input_power: number;
		active_power: number;
		reactive_power: number;
		power_factor: number;
		efficiency: number;
		temperature: number;
		pv_voltage_current: number[];
	}

	let complete_data: { weather: WeatherData; trackers: TrackerData[]; inverters: InverterData[] } = undefined;
	async function load_data() {
		const weather: WeatherData = await invoke('request_weather_data');
		const trackers: TrackerData[] = await invoke('request_trackers_data');
		const inverters: InverterData[] = await invoke('request_inverters_data');
		complete_data = { weather, trackers, inverters };
		console.log(complete_data);
	};
	load_data();
</script>

{#if complete_data}
	<main class="container">
		{#if complete_data.weather}
			<h1 class="text-2xl font-bold">Weather station</h1>
			{#each Object.entries(complete_data.weather) as [key, value]}
				<p>{key}: {value}</p>
			{/each}
		{/if}
		<h1 class="text-2xl font-bold">Trackers data</h1>
		{#each complete_data.trackers as tracker, i}
			<h2 class="text-xl">Tracker {i + 1}</h2>
			{#each Object.entries(tracker) as [key, value]}
				<p>{key}: {value}</p>
			{/each}
		{/each}
		<h1 class="text-2xl font-bold">Inverters data</h1>
		{#each complete_data.inverters as inverter, i}
			<h2 class="text-xl">Inverter {i + 1}</h2>
			{#each Object.entries(inverter) as [key, value]}
				<p>{key}: {value}</p>
			{/each}
		{/each}
	</main>
{:else}
	<h1>Loading...</h1>
{/if}
