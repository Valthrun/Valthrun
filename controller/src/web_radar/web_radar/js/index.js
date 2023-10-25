
var ws = new WebSocket('ws://localhost:6969/ws');

ws.onopen = function() {
    console.log('Connected to the WebSocket');
};

ws.onmessage = function(event) {
    // Assuming you parse the event.data to get an array of player data
    var players = JSON.parse(event.data);

    // Remove all existing player dots
    var existingDots = document.querySelectorAll('.player-dot');
    existingDots.forEach(dot => dot.remove());

    console.log('Data received!');
    // Add and position a dot for each player
    players.forEach(player => {
        var playerDot = addPlayerDot(player.team_id);

        // var rotation = player.rotation;
        var x = player.position[0]; // as a percentage of the map width
        var y = player.position[1]; // as a percentage of the map height

        // Rotate and position the player dot
        // playerDot.style.transform = `translate(-50%, -50%) rotate(${rotation}deg)`;
        var size = 2048 * 3.66;
        var offset = { x: 3240, y: 3410 };
        x = (x + offset.x) / size * 100;
        y = (y + offset.y) / size * 100;
        console.log('Player pos:', x, y);
        playerDot.style.left = `${x}%`;
        playerDot.style.top = `${Math.abs(y - 100)}%`;
    });

};

ws.onclose = function(event) {
    console.log('WebSocket closed:', event.code, event.reason);
};

ws.onerror = function(error) {
    console.log('WebSocket Error:', error);
};

function addPlayerDot(teamID) {
    // Create a new image element
    var playerDot = document.createElement('img');
    console.log(teamID);
    if (teamID === 3)
    {
        playerDot.src = 'images/blue_dot.png';
        playerDot.alt = 'Player';
        playerDot.className = 'player-dot';
    }
    else
    {
        playerDot.src = 'images/yellow_dot.png';
        playerDot.alt = 'Player';
        playerDot.className = 'player-dot';
    }

    // Append the player dot to the map container
    var mapContainer = document.querySelector('.map-container');
    mapContainer.appendChild(playerDot);

    return playerDot; // Return the created element for further manipulation
}