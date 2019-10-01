import React from 'react';
import SwipeableViews from 'react-swipeable-views';
import { Grid, Typography, makeStyles, Paper, Tabs, Tab, Box } from '@material-ui/core';

import EthereumToSubstrate from '~components/EthereumToSubstrate';
import SubstrateToEthereum from '~components/SubstrateToEthereum';

const useStyles = makeStyles(theme => ({
  root: {
    padding: theme.spacing(3),
    maxWidth: 1200,
    margin: '0 auto',
  }
}));

function App() {
  const classes = useStyles();

  const [value, setValue] = React.useState(0);

  const handleChange = (_event: React.ChangeEvent<{}>, newValue: number) => {
    setValue(newValue);
  };

  const handleChangeIndex = (index: number) => {
    setValue(index);
  };

  return (
    <Grid container spacing={3} className={classes.root}>
      <Grid item xs={12}>
        <Typography variant="h2" align="center" gutterBottom>Ethereum DAI {'<-->'} AkropolisOS Bridge</Typography>
      </Grid>
      <Grid item xs={12}>
        <Paper>
          <Tabs
            value={value}
            onChange={handleChange}
            indicatorColor="primary"
            textColor="primary"
            variant="fullWidth"
          >
            <Tab label="Ethereum to Substrate" />
            <Tab label="Substrate to Ethereum" />
          </Tabs>
        </Paper>
      </Grid>
      <SwipeableViews
        index={value}
        onChangeIndex={handleChangeIndex}
      >
        <Box p={2}>
          <EthereumToSubstrate />
        </Box>
        <Box p={2}>
          <SubstrateToEthereum />
        </Box>
      </SwipeableViews>
    </Grid>
  );
}

export default App;
