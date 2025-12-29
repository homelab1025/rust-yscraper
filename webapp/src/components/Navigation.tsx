import { Link } from 'react-router-dom';
import { Tabs, Tab, Box } from '@mui/material';
import { useLocation } from 'react-router-dom';

const Navigation = () => {
  const location = useLocation();
  const currentPath = location.pathname === '/' ? '/links' : location.pathname;

  return (
    <Box sx={{ borderBottom: 1, borderColor: 'divider', mb: 2 }}>
      <Tabs value={currentPath}>
        <Tab 
          label="Links" 
          value="/links" 
          component={Link} 
          to="/links" 
        />
        <Tab 
          label="About" 
          value="/about" 
          component={Link} 
          to="/about" 
        />
      </Tabs>
    </Box>
  );
};

export default Navigation;
