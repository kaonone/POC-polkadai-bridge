import * as React from 'react';
import Radio from '@material-ui/core/Radio';
import { FieldRenderProps } from 'react-final-form';

type Props = FieldRenderProps<string, HTMLInputElement>;

function RadioWrapper ({
  input: { checked, value, name, onChange, ...restInput },
  ...rest
}: Props): React.ReactElement<Props> {
  delete rest.meta;

  return (
    <Radio
      {...rest}
      name={name}
      inputProps={restInput}
      onChange={onChange}
      checked={checked}
      value={value}
    />
  );
}

export default RadioWrapper;
