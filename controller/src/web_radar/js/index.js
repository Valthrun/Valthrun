
var ws = new WebSocket('ws://localhost:6969/ws');

ws.onopen = function() {
    console.log('Connected to the WebSocket');
};

ws.onmessage = function(event) {
    // Assuming you parse the event.data to get an array of player data
    /*var players = JSON.parse(event.data);

    // Remove all existing player dots
    var existingDots = document.querySelectorAll('.player-dot');
    existingDots.forEach(dot => dot.remove());

    // Add and position a dot for each player
    players.forEach(player => {
        var playerDot = addPlayerDot();

        // var rotation = player.rotation;
        var x = player.position.x; // as a percentage of the map width
        var y = player.position.z; // as a percentage of the map height

        // Rotate and position the player dot
        // playerDot.style.transform = `translate(-50%, -50%) rotate(${rotation}deg)`;
        var size = 2048 * 5.02;
        var offset = (3240, 3410);
        x = (x + offset.x) / size * 100;
        y = (y + offset.y) / size * 100;
        playerDot.style.left = `${x}%`;
        playerDot.style.top = `${y}%`;
    });*/
};

ws.onclose = function(event) {
    console.log('WebSocket closed:', event.code, event.reason);
};

ws.onerror = function(error) {
    console.log('WebSocket Error:', error);
};

function addPlayerDot() {
    // Create a new image element
    var playerDot = document.createElement('img');
    playerDot.src = '/path_to_player_dot.png';
    playerDot.alt = 'Player';
    playerDot.className = 'player-dot';

    // Append the player dot to the map container
    var mapContainer = document.querySelector('.map-container');
    mapContainer.appendChild(playerDot);

    return playerDot; // Return the created element for further manipulation
}