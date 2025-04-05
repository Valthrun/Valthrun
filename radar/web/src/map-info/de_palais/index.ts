import { LoadedMap } from "..";
import OfficialDefault from "./map_style_cs2.png";
import OfficialHigher1 from "./map_style_cs2_higher1.png";
import OfficialHigher2 from "./map_style_cs2_higher2.png";

export default {
    mapName: "de_palais",
    displayName: "Palais",

    pos_x: -1353.2374, // upper left world coordinate
    pos_y: 2044.8105,
    scale: 2.8003397,

    verticalSections: [
        {
            name: "default",
            altitudeMax: -16384,
            altitudeMin: -47.99997,
        },
        {
            name: "lower",
            altitudeMax: 47.99997,
            altitudeMin: 214.00027,
        },
		{
			name: "higher",
			altitudeMax: 214.00027,
			altitudeMin: 16384,
		}
    ],

    mapStyles: [
        {
            name: "Official",
            map: {
                default: OfficialDefault,
				lower: OfficialHigher1,
				higher: OfficialHigher2
            }
        }
    ]
} satisfies LoadedMap;
