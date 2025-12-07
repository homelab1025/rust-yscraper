import React from 'react';
import ListComments from './components/ListComments';

export default function App() {
  return (
    <div className="page">
      <header className="hero">
        <i className="hero-icon" aria-hidden="true" />
        <h1>YScraper Dashboard</h1>
      </header>

      <main className="content">
        <ListComments />
      </main>
    </div>
  );
}
