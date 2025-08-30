# KZRK Game API Documentation

A RESTful API for the KZRK trading game. Players can create games, travel between airports, trade cargo, and manage their fuel.

## Base URL

```
http://127.0.0.1:3000
```

## Authentication

No authentication required for this version.

## Endpoints

### Health Check

**GET** `/health`

Check if the API is running.

**Response:**
```json
{
  "message": "KZRK Game API is running"
}
```

### Game Management

#### Create New Game

**POST** `/game`

Create a new game session.

**Request Body:**
```json
{
  "player_name": "string",
  "starting_money": 5000,          // optional, default: 5000
  "starting_airport": "JFK"        // optional, default: "JFK"
}
```

**Response:**
```json
{
  "session_id": "uuid",
  "player_name": "string", 
  "game_state": { ... }            // Full game state
}
```

#### Get Game State

**GET** `/game/{session_id}`

Get the current state of a game session.

**Response:**
```json
{
  "player": {
    "name": "string",
    "money": 5000,
    "current_airport": "JFK",
    "fuel": 66,
    "max_fuel": 200,
    "cargo_inventory": {},
    "cargo_weight": 0,
    "max_cargo_weight": 1000,
    "fuel_efficiency": 15.0
  },
  "current_market": {
    "airport_id": "JFK",
    "airport_name": "New York JFK",
    "fuel_price": 80,
    "cargo_prices": {
      "electronics": 400,
      "food": 120
    }
  },
  "available_destinations": [
    {
      "airport_id": "LAX", 
      "airport_name": "Los Angeles LAX",
      "distance": 3944.4,
      "fuel_required": 263,
      "can_travel": false,
      "fuel_price": 70
    }
  ],
  "active_events": [],
  "statistics": {
    "total_revenue": 0,
    "total_expenses": 0,
    "net_profit": 0,
    "cargo_trades": 0,
    "fuel_purchased": 0,
    "distances_traveled": 0.0,
    "airports_visited": [],
    "best_single_trade": 0,
    "most_profitable_cargo": "",
    "efficiency_score": 0.0
  },
  "turn_number": 1
}
```

### Game Actions

#### Travel

**POST** `/game/{session_id}/travel`

Travel to another airport.

**Request Body:**
```json
{
  "destination": "LAX"
}
```

**Response:**
```json
{
  "success": true,
  "message": "Traveled to Los Angeles LAX (LAX)",
  "fuel_consumed": 263,
  "new_location": "LAX",
  "game_state": { ... }            // Updated game state
}
```

#### Trade Cargo

**POST** `/game/{session_id}/trade`

Buy or sell cargo at the current airport.

**Request Body:**
```json
{
  "cargo_type": "electronics",
  "quantity": 5,
  "action": "Buy"                  // "Buy" or "Sell"
}
```

**Response:**
```json
{
  "success": true,
  "message": "Successfully Bought 5 units of electronics",
  "transaction_amount": 2000,
  "new_money": 3000,
  "new_inventory": {
    "electronics": 5
  },
  "game_state": { ... }            // Updated game state
}
```

#### Buy Fuel

**POST** `/game/{session_id}/fuel`

Purchase fuel at the current airport.

**Request Body:**
```json
{
  "quantity": 50
}
```

**Response:**
```json
{
  "success": true,
  "message": "Purchased 50 units of fuel for $4000",
  "cost": 4000,
  "new_fuel": 116,
  "new_money": 1000,
  "game_state": { ... }            // Updated game state
}
```

### Reference Data

#### Get Airports

**GET** `/airports`

Get list of all available airports.

**Response:**
```json
[
  {
    "id": "JFK",
    "name": "New York JFK", 
    "latitude": 40.6413,
    "longitude": -73.7781
  }
]
```

#### Get Cargo Types

**GET** `/cargo`

Get list of all cargo types.

**Response:**
```json
[
  {
    "id": "electronics",
    "name": "Electronics",
    "base_price": 500,
    "weight": 1,
    "volatility": 0.4
  }
]
```

## Error Handling

All endpoints return appropriate HTTP status codes:

- **200 OK** - Successful request
- **400 Bad Request** - Invalid request data
- **404 Not Found** - Game session not found
- **500 Internal Server Error** - Server error

Error responses follow this format:
```json
{
  "error": "ErrorType",
  "message": "Human readable error message",
  "details": null
}
```

## Game Mechanics

- **Fuel Consumption**: Travel between airports consumes fuel based on distance and fuel efficiency
- **Cargo Weight**: Players have limited cargo capacity measured in weight units  
- **Market Prices**: Cargo prices vary between airports and change over time
- **Turn-based**: Each action (travel, trade, fuel purchase) advances the game turn
- **Statistics**: Game tracks player performance metrics

## Usage Examples

### Complete Game Flow

1. **Create Game**
   ```bash
   curl -X POST http://127.0.0.1:3000/game \
     -H "Content-Type: application/json" \
     -d '{"player_name": "TestPlayer"}'
   ```

2. **Check Available Destinations**
   ```bash
   curl http://127.0.0.1:3000/game/{session_id}
   ```

3. **Buy Fuel for Travel**
   ```bash
   curl -X POST http://127.0.0.1:3000/game/{session_id}/fuel \
     -H "Content-Type: application/json" \
     -d '{"quantity": 100}'
   ```

4. **Buy Cargo**
   ```bash
   curl -X POST http://127.0.0.1:3000/game/{session_id}/trade \
     -H "Content-Type: application/json" \
     -d '{"cargo_type": "electronics", "quantity": 3, "action": "Buy"}'
   ```

5. **Travel to Another Airport**
   ```bash
   curl -X POST http://127.0.0.1:3000/game/{session_id}/travel \
     -H "Content-Type: application/json" \
     -d '{"destination": "LAX"}'
   ```

6. **Sell Cargo for Profit**
   ```bash
   curl -X POST http://127.0.0.1:3000/game/{session_id}/trade \
     -H "Content-Type: application/json" \
     -d '{"cargo_type": "electronics", "quantity": 3, "action": "Sell"}'
   ```