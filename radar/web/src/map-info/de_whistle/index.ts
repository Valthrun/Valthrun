import { LoadedMap } from "..";
import OfficialDefault from "./map_style_cs2.png";

export default {
    mapName: "de_whistle",
    displayName: "Whistle",

    pos_x: -1825.6, // upper left world coordinate
    pos_y: 1104.82,
    scale: 2.8,

    verticalSections: [
        {
            name: "default",
            altitudeMax: 10000,
            altitudeMin: -10000,
        },
    ],

    mapStyles: [
{
            name: "Official",
            map: {
                default: OfficialDefault,
            }
        }
    ]
} satisfies LoadedMap;
