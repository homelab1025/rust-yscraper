import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import { ThemeProvider, createTheme, CssBaseline, Container } from '@mui/material';
import Navigation from './components/Navigation';
import LinkManagementPage from './pages/LinkManagementPage';
import AboutPage from './pages/AboutPage';

const theme = createTheme({
  palette: {
    mode: 'light',
    primary: {
      main: '#ff6600', // Hacker News orange
    },
  },
});

function App() {
  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <Router>
        <Navigation />
        <Container component="main" sx={{ py: 4 }}>
          <Routes>
            <Route path="/links" element={<LinkManagementPage />} />
            <Route path="/about" element={<AboutPage />} />
            <Route path="/" element={<Navigate to="/links" replace />} />
          </Routes>
        </Container>
      </Router>
    </ThemeProvider>
  );
}

export default App;
