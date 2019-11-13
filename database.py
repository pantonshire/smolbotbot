import initdata

from sqlalchemy import create_engine
from sqlalchemy.orm import sessionmaker


db_connection_data = initdata.read_lines("data/.db")
uri = db_connection_data[0]

engine = create_engine(uri)

del uri, db_connection_data

Session = sessionmaker(bind=engine)


def accessdb(create_transaction, *args):
    session = Session()
    try:
        create_transaction(session, *args)
        session.commit()
    except:
        session.rollback()
        raise
    finally:
        session.close()
        session = None
