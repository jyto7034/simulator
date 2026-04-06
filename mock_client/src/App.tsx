import { useState } from 'react'
import './App.css'

type Screen = 'shop' | 'event' | 'pve' | 'bag'

type UnitCell = {
  label: string
  accent?: 'ally' | 'enemy' | 'boss' | 'empty'
}

type StageCard = {
  title: string
  subtitle: string
  meta: string
}

type ItemShape = 'square' | 'diamond'

type InventoryItem = {
  code: string
  name: string
  effect: string
  shape: ItemShape
}

type ReserveUnit = {
  name: string
  role: string
  note: string
}

const screenLabels: Record<Screen, string> = {
  shop: 'Shop',
  event: 'Event',
  pve: 'PvE',
  bag: 'Bag',
}

const inventoryItems: InventoryItem[] = [
  { code: 'HK', name: 'Hook', effect: 'Pulls the first struck enemy slightly toward the frontline.', shape: 'diamond' },
  { code: 'GR', name: 'Grail', effect: 'Restore health to the wearer after each takedown.', shape: 'square' },
  { code: 'CH', name: 'Charm', effect: 'Raises skill trigger chance for the equipped unit.', shape: 'diamond' },
  { code: 'SH', name: 'Shell', effect: 'Gain a barrier at battle start.', shape: 'square' },
  { code: 'SP', name: 'Spur', effect: 'Increases move speed and opening attack timing.', shape: 'diamond' },
  { code: 'TH', name: 'Thread', effect: 'Boosts support effects on adjacent allies.', shape: 'square' },
]

const reserveUnits: ReserveUnit[] = [
  { name: 'Librarian', role: 'Backline', note: 'Reserve damage dealer' },
  { name: 'Fixer', role: 'Frontline', note: 'Swaps into tank lane' },
  { name: 'Sniper', role: 'Ranged', note: 'Pairs with Hook and Spur' },
  { name: 'Warden', role: 'Guard', note: 'Shield utility option' },
]

const shopCards: StageCard[] = [
  { title: 'Crimson Saber', subtitle: 'Frontline weapon', meta: '12' },
  { title: 'Moth Lantern', subtitle: 'Resonance trinket', meta: '8' },
  { title: 'Velvet Plate', subtitle: 'Guard armor', meta: '15' },
  { title: 'Tea Vial', subtitle: 'Recovery item', meta: '6' },
]

const eventCards: StageCard[] = [
  { title: 'Black Market', subtitle: 'Shop encounter', meta: 'Trade' },
  { title: 'Strange Guest', subtitle: 'Reward encounter', meta: 'Curio' },
  { title: 'Suppression', subtitle: 'Combat encounter', meta: 'Risk' },
]

const enemyRows: UnitCell[][] = [
  [
    { label: '', accent: 'empty' },
    { label: '', accent: 'empty' },
    { label: 'Hunter', accent: 'enemy' },
    { label: '', accent: 'empty' },
    { label: 'Moth', accent: 'enemy' },
    { label: '', accent: 'empty' },
    { label: '', accent: 'empty' },
    { label: 'Claw', accent: 'enemy' },
  ],
  [
    { label: '', accent: 'empty' },
    { label: '', accent: 'empty' },
    { label: 'Fixer', accent: 'enemy' },
    { label: '', accent: 'empty' },
    { label: 'Abno', accent: 'boss' },
    { label: '', accent: 'empty' },
    { label: '', accent: 'empty' },
  ],
  [
    { label: '', accent: 'empty' },
    { label: 'Drone', accent: 'enemy' },
    { label: '', accent: 'empty' },
    { label: '', accent: 'empty' },
    { label: '', accent: 'empty' },
    { label: 'Hound', accent: 'enemy' },
    { label: '', accent: 'empty' },
  ],
  [
    { label: '', accent: 'empty' },
    { label: '', accent: 'empty' },
    { label: '', accent: 'empty' },
    { label: '', accent: 'empty' },
    { label: '', accent: 'empty' },
    { label: '', accent: 'empty' },
    { label: '', accent: 'empty' },
  ],
]

const playerRows: UnitCell[][] = [
  [
    { label: '', accent: 'empty' },
    { label: '', accent: 'empty' },
    { label: 'Lob. Corp', accent: 'ally' },
    { label: '', accent: 'empty' },
    { label: 'Liu', accent: 'ally' },
    { label: '', accent: 'empty' },
    { label: '', accent: 'empty' },
    { label: 'Blade', accent: 'ally' },
  ],
  [
    { label: '', accent: 'empty' },
    { label: '', accent: 'empty' },
    { label: 'Tank', accent: 'ally' },
    { label: '', accent: 'empty' },
    { label: 'Support', accent: 'ally' },
    { label: '', accent: 'empty' },
    { label: '', accent: 'empty' },
  ],
  [
    { label: 'Guard', accent: 'ally' },
    { label: '', accent: 'empty' },
    { label: '', accent: 'empty' },
    { label: 'Page', accent: 'ally' },
    { label: '', accent: 'empty' },
    { label: '', accent: 'empty' },
    { label: '', accent: 'empty' },
  ],
  [
    { label: '', accent: 'empty' },
    { label: '', accent: 'empty' },
    { label: '', accent: 'empty' },
    { label: '', accent: 'empty' },
    { label: '', accent: 'empty' },
    { label: '', accent: 'empty' },
    { label: '', accent: 'empty' },
  ],
]

function App() {
  const [screen, setScreen] = useState<Screen>('shop')

  return (
    <main className={`app screen-${screen}`}>
      <div className="viewport-frame">
        <div className="atmosphere atmosphere-left" aria-hidden="true" />

        <section className="mock-shell">
          <section className="table-shell">
            <header className="status-bar">
              <div className="phase-strip">
                <span>Phase II</span>
                <span>Qliphoth 2</span>
                <span>Enk 12</span>
                <span>Gold 87</span>
              </div>

              <div className="stage-switcher" aria-label="Mock screens">
                {(['shop', 'event', 'pve', 'bag'] as Screen[]).map((key) => (
                  <button
                    key={key}
                    type="button"
                    className={key === screen ? 'is-active' : undefined}
                    onClick={() => setScreen(key)}
                  >
                    {screenLabels[key]}
                  </button>
                ))}
              </div>
            </header>

            <section className="board-surface">
              <div className="board-half opponent-half">
                {screen === 'shop' && <ShopStage />}
                {screen === 'event' && <EventStage />}
                {screen === 'pve' && <PveStage />}
                {screen === 'bag' && <BagStage />}
              </div>

              <div className="frontline" role="presentation">
                <span>Frontline</span>
              </div>

              <div className="board-half player-half">
                <div className="field-caption">
                  <span className="eyebrow">My Board</span>
                  <strong>Player Formation</strong>
                </div>
                <BoardRows rows={playerRows} />
              </div>
            </section>
          </section>
        </section>

        <div className="atmosphere atmosphere-right" aria-hidden="true" />
      </div>
    </main>
  )
}

function ShopStage() {
  return (
    <div className="stage-scene shop-scene">
      <div className="shop-canopy" aria-hidden="true" />
      <div className="shop-shelf">
        {shopCards.map((card) => (
          <article key={card.title} className="shop-item">
            <span className="slot-index">Cost {card.meta}</span>
            <strong>{card.title}</strong>
            <span>{card.subtitle}</span>
          </article>
        ))}
      </div>
      <div className="selection-plaque">
        <span className="slot-index">Selected Item</span>
        <strong>Crimson Saber</strong>
        <span>Frontline weapon staged in the enemy-side market space.</span>
      </div>
    </div>
  )
}

function EventStage() {
  return (
    <div className="stage-scene event-scene">
      <div className="event-pedestals">
        {eventCards.map((card, index) => (
          <article key={card.title} className={`event-offer ${index === 1 ? 'is-featured' : ''}`}>
            <span className="slot-index">{card.meta}</span>
            <strong>{card.title}</strong>
            <span>{card.subtitle}</span>
          </article>
        ))}
      </div>
      <div className="selection-plaque centered">
        <span className="slot-index">Selected Event</span>
        <strong>Strange Guest</strong>
        <span>The decision stays on the opponent stage instead of becoming a fullscreen modal.</span>
      </div>
    </div>
  )
}

function PveStage() {
  return (
    <div className="stage-scene pve-scene">
      <div className="encounter-banner">
        <span className="slot-index">Suppression Target</span>
        <strong>Abnormality Wing</strong>
        <span>Risk WAW • Reward 3</span>
      </div>
      <BoardRows rows={enemyRows} enemy />
    </div>
  )
}

function BagStage() {
  return (
    <div className="stage-scene bag-scene">
      <section className="bag-panel">
        <div className="field-caption">
          <span className="eyebrow">Reserve Units</span>
          <strong>Waiting Roster</strong>
        </div>
        <div className="reserve-grid">
          {reserveUnits.map((unit) => (
            <article key={unit.name} className="reserve-card">
              <strong>{unit.name}</strong>
              <span>{unit.role}</span>
              <span className="slot-copy">{unit.note}</span>
            </article>
          ))}
        </div>
      </section>

      <section className="bag-panel">
        <div className="field-caption">
          <span className="eyebrow">Item Bag</span>
          <strong>Loose Equipment</strong>
        </div>
        <div className="bag-item-grid">
          {inventoryItems.map((item, index) => (
            <div key={item.name} className="inventory-slot bag-slot">
              <button type="button" className="inventory-button" aria-label={item.name}>
                <span className={`inventory-emblem shape-${item.shape}`}>
                  <span>{item.code}</span>
                </span>
              </button>

              <div className="inventory-tooltip" role="tooltip">
                <span className="slot-index">I0{index + 1}</span>
                <strong>{item.name}</strong>
                <span className="slot-copy">{item.effect}</span>
              </div>
            </div>
          ))}
        </div>
      </section>
    </div>
  )
}

function BoardRows({ rows, enemy = false }: { rows: UnitCell[][]; enemy?: boolean }) {
  return (
    <div className={`board-grid ${enemy ? 'enemy-board' : 'player-board'}`}>
      {rows.map((row, rowIndex) => (
        <div key={`row-${rowIndex}`} className="board-row">
          {row.map((cell, cellIndex) => (
            <article
              key={`cell-${rowIndex}-${cellIndex}`}
              className={`board-cell accent-${cell.accent ?? 'empty'}`}
            >
              <span>{cell.label}</span>
            </article>
          ))}
        </div>
      ))}
    </div>
  )
}

export default App
